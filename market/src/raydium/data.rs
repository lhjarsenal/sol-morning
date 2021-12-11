use crate::market;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use std::fs;
use std::collections::HashMap;
use solana_program::pubkey::Pubkey;
use rust_decimal::prelude::FromStr;
use market::{MarketPool, MarketOptMap, MarketType};

const RAYDIUM_MARKET: &str = "raydium";
const RAYDIUM_PROGRAM_ID: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";

#[derive(Serialize, Deserialize, Debug)]
pub struct RawMarketPool {
    pub id: String,
    pub quote_mint: String,
    pub base_mint: String,
    pub market_quote_vault: String,
    pub market_base_vault: String,
}

pub fn load_data_from_file(quote_mint: &String, base_mint: &String) -> Result<MarketOptMap> {
    let market_main_path = "./raydium_pool.json".to_string();

    let raw_info = fs::read_to_string(market_main_path).expect("Error read file");
    let vec: Vec<RawMarketPool> = serde_json::from_str(&raw_info)?;

    let mut quote_map = HashMap::new();
    let mut base_map = HashMap::new();

    for pool in &vec {
        if pool.quote_mint.eq(quote_mint) {
            let market_pool = MarketPool {
                pool_key: Pubkey::from_str(&pool.id).unwrap(),
                quote_mint_key: Pubkey::from_str(&pool.quote_mint).unwrap(),
                base_mint_key: Pubkey::from_str(&pool.base_mint).unwrap(),
                quote_value_key: Pubkey::from_str(&pool.market_quote_vault).unwrap(),
                base_value_key: Pubkey::from_str(&pool.market_base_vault).unwrap(),
                is_quote_to_base: true,
            };
            quote_map.insert(pool.base_mint.clone(), market_pool);
        }
        if pool.base_mint.eq(quote_mint) {
            let market_pool = MarketPool {
                pool_key: Pubkey::from_str(&pool.id).unwrap(),
                quote_mint_key: Pubkey::from_str(&pool.quote_mint).unwrap(),
                base_mint_key: Pubkey::from_str(&pool.base_mint).unwrap(),
                quote_value_key: Pubkey::from_str(&pool.market_quote_vault).unwrap(),
                base_value_key: Pubkey::from_str(&pool.market_base_vault).unwrap(),
                is_quote_to_base: false,
            };
            quote_map.insert(pool.quote_mint.clone(), market_pool);
        }
        if pool.quote_mint.eq(base_mint) {
            let market_pool = MarketPool {
                pool_key: Pubkey::from_str(&pool.id).unwrap(),
                quote_mint_key: Pubkey::from_str(&pool.quote_mint).unwrap(),
                base_mint_key: Pubkey::from_str(&pool.base_mint).unwrap(),
                quote_value_key: Pubkey::from_str(&pool.market_quote_vault).unwrap(),
                base_value_key: Pubkey::from_str(&pool.market_base_vault).unwrap(),
                is_quote_to_base: false,
            };
            base_map.insert(pool.base_mint.clone(), market_pool);
        }
        if pool.base_mint.eq(base_mint) {
            let market_pool = MarketPool {
                pool_key: Pubkey::from_str(&pool.id).unwrap(),
                quote_mint_key: Pubkey::from_str(&pool.quote_mint).unwrap(),
                base_mint_key: Pubkey::from_str(&pool.base_mint).unwrap(),
                quote_value_key: Pubkey::from_str(&pool.market_quote_vault).unwrap(),
                base_value_key: Pubkey::from_str(&pool.market_base_vault).unwrap(),
                is_quote_to_base: true,
            };
            base_map.insert(pool.quote_mint.clone(), market_pool);
        }
    }

    Ok(MarketOptMap {
        market_type: MarketType::Raydium(RAYDIUM_MARKET.to_string().clone(), RAYDIUM_PROGRAM_ID.to_string().clone()),
        quote_mint: quote_mint.clone(),
        base_mint: base_mint.clone(),
        quote_map,
        base_map,
    })
}

