use crate::api;
use crate::opt_core;
use crate::node_client;
use market::pool::{PoolInfo, PoolResponse};
use market::{raydium};
use serde::{Serialize, Deserialize};
use solana_program::pubkey::Pubkey;
use solana_sdk::{commitment_config::CommitmentConfig, account::Account};
use solana_client::rpc_client::RpcClient;
use node_client::NetworkType;
use market::market::MarketType::*;
use anyhow::Result;
use market::raydium::stats::AmmInfo;
use opt_core::convert_to_info;
use api::load_token_data_from_file;
use spl_token_swap::processor::Processor;
use api::TokenAddr;
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use bytemuck::__core::ops::{Add, Mul, Div};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct PoolRequest {
    pub market: Option<String>,
    pub token_mint_a: Option<String>,
    pub token_mint_b: Option<String>,
    pub lp_mint: Option<String>,
    pub need_rate: Option<bool>,
}

impl PoolRequest {
    pub fn load_data(&self) -> Vec<PoolInfo> {

        //参数校验
        if self.lp_mint.is_none() {
            if self.token_mint_a.is_none() || self.token_mint_b.is_none() {
                return vec![];
            }
        }

        let mut need_raydium = false;
        let mut _need_orca = false;
        match &self.market {
            Some(a) => {
                if a.eq("raydium") {
                    need_raydium = true;
                } else if a.eq("orca") {
                    _need_orca = true;
                } else {
                    return vec![];
                }
            }
            None => {
                need_raydium = true;
                _need_orca = true;
            }
        }


        let mut market_pool = vec![];

        if need_raydium {
            let raydium_pool = raydium::data::load_pool_from_file(self.lp_mint.clone(), self.token_mint_a.clone(),
                                                                  self.token_mint_b.clone());
            if raydium_pool.is_some() {
                market_pool.push(raydium_pool.unwrap());
            }
        }

        market_pool
    }
}

pub fn cal_rate(pools: &[PoolInfo]) -> Vec<PoolResponse> {

    //查询
    let token_main_path = "./token_mint.json".to_string();
    let tokens_adr = load_token_data_from_file(&token_main_path).expect("load token data fail");

    let mut keys: Vec<Pubkey> = vec![];
    for pool in pools {
        keys.push(pool.pool_key.clone());
        keys.push(pool.quote_value_key.clone());
        keys.push(pool.base_value_key.clone());
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

    let mut res = vec![];

    for pool in pools {
        match &pool.market_type {
            Raydium(_x, _y) => {
                let pool_info = cal_raydium(&account_map, &tokens_adr, &pool).unwrap();
                res.push(pool_info);
            }
            Orca(_x, _y) => {}
            Saber(_x, _y) => {}
            Swap(_x, _y) => {}
            Serum(_x, _y) => {}
        }
    }

    res
}

fn cal_raydium(account_map: &HashMap<String, Account>,
               token_map: &HashMap<String, TokenAddr>,
               pool: &PoolInfo) -> Result<PoolResponse> {
    let amount_in = 1.0;

    let pool_ac = account_map.get(&pool.pool_key.to_string()).unwrap();
    let mut pool_clone = pool_ac.clone();
    let pool_ac_info = convert_to_info(&pool.pool_key, &mut pool_clone);
    let pool_info = AmmInfo::load_amm_mut(&pool_ac_info, false).unwrap();

    let quote_ac = account_map.get(&pool.quote_value_key.to_string()).unwrap();
    let mut quote_clone = quote_ac.clone();
    let quote_ac_info = convert_to_info(&pool.quote_value_key, &mut quote_clone);
    let quote_info = Processor::unpack_token_account(&quote_ac_info, &quote_ac.owner).unwrap();

    let base_ac = account_map.get(&pool.base_value_key.to_string()).unwrap();
    let mut base_clone = base_ac.clone();
    let base_ac_info = convert_to_info(&pool.base_value_key, &mut base_clone);
    let base_info = Processor::unpack_token_account(&base_ac_info, &base_ac.owner).unwrap();

    let basic: i128 = 10;
    let quote_token = token_map.get(&pool.quote_mint_key.to_string()).unwrap();
    let base_token = token_map.get(&pool.base_mint_key.to_string()).unwrap();
    let quote_pow = basic.pow(quote_token.decimal as u32);
    let base_pow = basic.pow(base_token.decimal as u32);
    let quote_amount = Decimal::from(quote_info.amount);
    let base_amount = Decimal::from(base_info.amount);

    let from_amount = Decimal::from_f64(amount_in * (quote_pow as f64)).unwrap();
    let from_amount_with_fee = from_amount.mul(Decimal::from(pool_info.fees.swap_fee_denominator - pool_info.fees.swap_fee_numerator)).div(Decimal::from(pool_info.fees.swap_fee_denominator));
    let denominator = quote_amount.add(from_amount_with_fee);
    let amount_out = base_amount.mul(from_amount_with_fee).div(denominator);
    let mut amount_out_format = amount_out.div(Decimal::from(base_pow));
    amount_out_format.rescale(base_token.decimal as u32);

    let matket_type = pool.market_type.get_name();
    Ok(PoolResponse {
        market: matket_type.0,
        program_id: matket_type.1,
        pool_account: pool.pool_key.to_string(),
        quote_mint: pool.quote_mint_key.to_string(),
        base_mint: pool.base_mint_key.to_string(),
        lp_mint: pool.lp_mint_key.to_string(),
        quote_value: pool.quote_value_key.to_string(),
        base_value: pool.base_value_key.to_string(),
        rate: Some(amount_out_format.to_f64().unwrap()),
    })
}