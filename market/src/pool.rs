use std::collections::HashMap;
use crate::market::MarketType;
use serde::{Serialize, Deserialize};
use solana_program::pubkey::Pubkey;
use std::fs;
use crate::{orca, raydium};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PoolInfo {
    pub market_type: MarketType,
    pub pool_key: Pubkey,
    pub quote_mint_key: Pubkey,
    pub base_mint_key: Pubkey,
    pub lp_mint_key: Pubkey,
    pub quote_value_key: Pubkey,
    pub base_value_key: Pubkey,
    pub data: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PoolResponse {
    pub market: String,
    pub program_id: String,
    pub pool_account: String,
    pub quote_mint: String,
    pub base_mint: String,
    pub lp_mint: String,
    pub quote_value: String,
    pub base_value: String,
    pub rate: Option<f64>,
    pub data: HashMap<String, String>,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RawPool {
    pub market: String,
    pub pool_key: String,
    pub quote_mint: String,
    pub base_mint: String,
    pub lp_mint: String,
}

impl RawPool {
    pub fn load_all_pool_data(market: Option<String>) -> Vec<RawPool> {
        let mut vec = vec![];

        let mut need_raydium = false;
        let mut need_orca = false;
        match market {
            Some(name) => {
                if name.eq("raydium".trim()) {
                    need_raydium = true;
                } else if name.eq("orca".trim()) {
                    need_orca = true;
                } else {
                    return vec;
                }
            }
            None => {
                need_orca = true;
                need_raydium = true;
            }
        }

        if need_raydium {
            let raydium_pool_path = "./resource/pool/raydium.json".to_string();
            let raydium_raw = fs::read_to_string(raydium_pool_path).expect("Error read file");
            let raydium_vec: Vec<raydium::data::RawPoolInfo> = serde_json::from_str(&raydium_raw).unwrap();
            for raydium in &raydium_vec {
                vec.push(RawPool {
                    market: "raydium".to_string(),
                    pool_key: raydium.id.clone(),
                    quote_mint: raydium.quote_mint.clone(),
                    base_mint: raydium.base_mint.clone(),
                    lp_mint: raydium.lp_mint.clone(),
                })
            };
        }

        if need_orca {
            let orca_pool_path = "./resource/pool/orca.json".to_string();
            let orca_raw = fs::read_to_string(orca_pool_path).expect("Error read file");
            let orca_vec: Vec<orca::data::RawMarketPool> = serde_json::from_str(&orca_raw).unwrap();
            for orca in &orca_vec {
                vec.push(RawPool {
                    market: "orca".to_string(),
                    pool_key: orca.account.clone(),
                    quote_mint: orca.quote.mint.clone(),
                    base_mint: orca.base.mint.clone(),
                    lp_mint: orca.pool_mint.clone(),
                })
            };
        }

        vec
    }
}

