use crate::node_client::NetworkType;
use crate::opt_core;
use crate::response;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use std::fs;
use std::collections::HashMap;
use rust_decimal::prelude::FromStr;
use rocket_contrib::json::Json;
use market::{raydium, saber, orca};
use market;
use solana_program::{program_pack::Pack, pubkey::Pubkey};
use solana_program::{instruction::{AccountMeta, Instruction},
                     account_info::IntoAccountInfo,
};

use solana_sdk::{commitment_config::CommitmentConfig,
                 account::{Account, ReadableAccount, WritableAccount},
                 transaction::Transaction,
                 signature::Keypair,
                 signer::Signer,
};

use solana_client::{
    client_error::Result as ClientResult,
    rpc_client::RpcClient,
    rpc_config,
    rpc_filter,
};
use solana_client::rpc_response::RpcResult;

use bytemuck::__core::borrow::BorrowMut;
use opt_core::OptInitData;
use solana_program::account_info::AccountInfo;
use response::{OptResponse, OptRank};

use spl_token_swap::state::SwapV1;


#[derive(Debug, Serialize, Deserialize)]
pub struct ToDo {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub done: bool,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct OptRequest {
    pub amount_in: f32,
    pub quote_mint: String,
    pub base_mint: String,
    pub slippage: u32,
    pub exclude: Option<Vec<String>>,
}

impl OptRequest {
    pub fn load_data(&self) -> OptRank {

        //查询
        let token_main_path = "./token_mint.json".to_string();
        let tokens_adr = load_token_data_from_file(&token_main_path).expect("load token data fail");

        //key不支持返回错误 todo
        let quote_token = tokens_adr.get(&self.quote_mint).expect("pubKey not found");
        let base_token = tokens_adr.get(&self.base_mint).expect("pubKey not found");

        let markets = vec!["raydium", "orca", "saber", "swap", "serum"];

        let mut market_swap = vec![];

        let raydium_pool = raydium::data::load_data_from_file(&self.quote_mint, &self.base_mint).expect("load raydium data fail");
        let mut raydium_swap = raydium_pool.filer_swap().unwrap();

        let orca_pool = orca::data::load_data_from_file(&self.quote_mint, &self.base_mint).expect("load orca data fail");
        let mut orca_swap = orca_pool.filer_swap().unwrap();

        let saber_pool = saber::data::load_data_from_file(&self.quote_mint, &self.base_mint).expect("load orca data fail");
        let mut saber_swap = saber_pool.filer_swap().unwrap();

        market_swap.append(&mut raydium_swap);
        println!("market_swap={}", market_swap.len());
        market_swap.append(&mut orca_swap);
        println!("market_swap={}", market_swap.len());
        market_swap.append(&mut saber_swap);
        println!("market_swap={:?}", market_swap);

        let mut keys: Vec<Pubkey> = vec![];
        for swap in &market_swap {
            for step in &swap.step {
                keys.push(step.pool_key.clone());
                keys.push(step.quote_value_key.clone());
                keys.push(step.base_value_key.clone());
            }
        }

        let client = RpcClient::new(NetworkType::Mainnet.url().to_string());
        let commitment_config = CommitmentConfig::processed();
        let accounts: Vec<Option<Account>> = client.get_multiple_accounts_with_commitment(&keys, commitment_config).unwrap().value;

        let mut account_map = HashMap::new();
        for (index, value) in accounts.iter().enumerate() {
            let pubkey = &keys[index];
            match value {
                Some(account) => {
                    account_map.insert(pubkey.to_string(), account.clone());
                }
                None => {}
            }
        }

        //todo 暂时写死 50%拆单
        let route_percent = 0.5;
        let amount_in: f32 = self.amount_in.clone() * route_percent;

        let opt_init_data = OptInitData {
            amount_in,
            tokens_adr,
            account_map,
            swaps: market_swap,
        };
        let opt = opt_init_data.calculate().unwrap();

        OptRank {
            amount_out: self.amount_in,
            quote_mint: self.quote_mint.to_string(),
            base_mint: self.base_mint.to_string(),
            slippage: self.slippage,
            opt,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RawTokenAddr {
    pub symbol: String,
    pub address: String,
    pub decimals: u8,
    pub name: String,
    #[serde(rename = "logoURI")]
    pub icon_uri: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenAddr {
    pub name: String,
    pub mint: Pubkey,
    pub decimal: u8,
    pub description: String,
}

pub fn load_token_data_from_file(path: &String) -> Result<HashMap<String, TokenAddr>> {
    let raw_info = fs::read_to_string(path).expect("Error read file");
    let vec: Vec<RawTokenAddr> = serde_json::from_str(&raw_info)?;
    let res: HashMap<String, TokenAddr> = vec
        .iter()
        .map(|x| {
            let key = x.address.clone();
            (key, TokenAddr {
                name: (x.symbol).to_string(),
                mint: Pubkey::from_str(&x.address).unwrap(),
                decimal: x.decimals,
                description: (x.name).to_string(),
            })
        })
        .collect();
    Ok(res)
}
