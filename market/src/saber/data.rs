use crate::market;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use std::fs;
use std::collections::HashMap;
use solana_program::pubkey::Pubkey;
use rust_decimal::prelude::FromStr;
use market::{MarketPool, MarketOptMap, MarketType};

const SABER_MARKET: &str = "saber";
const SABER_PROGRAM_ID: &str = "SSwpkEEcbUqx4vtoEByFjSkhKdCT862DNVb52nZg1UZ";

#[derive(Serialize, Deserialize, Debug)]
pub struct RawMarketPool {
    pub account: String,
    pub quote: RawMarketToken,
    pub base: RawMarketToken,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RawMarketToken {
    pub mint: String,
    pub reserves: String,
}

pub fn load_data_from_file(quote_mint: &String, base_mint: &String) -> Result<MarketOptMap> {
    let market_main_path = "./saber_pool.json".to_string();

    let raw_info = fs::read_to_string(market_main_path).expect("Error read file");
    let vec: Vec<RawMarketPool> = serde_json::from_str(&raw_info)?;

    let mut quote_map = HashMap::new();
    let mut base_map = HashMap::new();

    for pool in &vec {
        let mut data = HashMap::new();
        data.insert("poolMint".to_string(), "pool_mint".to_string());
        data.insert("feeAccount".to_string(), "fee_account".to_string());

        if pool.quote.mint.eq(quote_mint) {
            let market_pool = MarketPool {
                pool_key: Pubkey::from_str(&pool.account).unwrap(),
                quote_mint_key: Pubkey::from_str(&pool.quote.mint).unwrap(),
                base_mint_key: Pubkey::from_str(&pool.base.mint).unwrap(),
                quote_value_key: Pubkey::from_str(&pool.quote.reserves).unwrap(),
                base_value_key: Pubkey::from_str(&pool.base.reserves).unwrap(),
                is_quote_to_base: true,
                data: data.clone(),
            };
            quote_map.insert(pool.base.mint.clone(), market_pool);
        }
        if pool.base.mint.eq(quote_mint) {
            let market_pool = MarketPool {
                pool_key: Pubkey::from_str(&pool.account).unwrap(),
                quote_mint_key: Pubkey::from_str(&pool.quote.mint).unwrap(),
                base_mint_key: Pubkey::from_str(&pool.base.mint).unwrap(),
                quote_value_key: Pubkey::from_str(&pool.quote.reserves).unwrap(),
                base_value_key: Pubkey::from_str(&pool.base.reserves).unwrap(),
                is_quote_to_base: false,
                data: data.clone(),
            };
            quote_map.insert(pool.quote.mint.clone(), market_pool);
        }
        if pool.quote.mint.eq(base_mint) {
            let market_pool = MarketPool {
                pool_key: Pubkey::from_str(&pool.account).unwrap(),
                quote_mint_key: Pubkey::from_str(&pool.quote.mint).unwrap(),
                base_mint_key: Pubkey::from_str(&pool.base.mint).unwrap(),
                quote_value_key: Pubkey::from_str(&pool.quote.reserves).unwrap(),
                base_value_key: Pubkey::from_str(&pool.base.reserves).unwrap(),
                is_quote_to_base: false,
                data: data.clone(),
            };
            base_map.insert(pool.base.mint.clone(), market_pool);
        }
        if pool.base.mint.eq(base_mint) {
            let market_pool = MarketPool {
                pool_key: Pubkey::from_str(&pool.account).unwrap(),
                quote_mint_key: Pubkey::from_str(&pool.quote.mint).unwrap(),
                base_mint_key: Pubkey::from_str(&pool.base.mint).unwrap(),
                quote_value_key: Pubkey::from_str(&pool.quote.reserves).unwrap(),
                base_value_key: Pubkey::from_str(&pool.base.reserves).unwrap(),
                is_quote_to_base: true,
                data: data.clone(),
            };
            base_map.insert(pool.quote.mint.clone(), market_pool);
        }
    }

    Ok(MarketOptMap {
        market_type: MarketType::Saber(SABER_MARKET.to_string().clone(), SABER_PROGRAM_ID.to_string().clone()),
        quote_mint: quote_mint.clone(),
        base_mint: base_mint.clone(),
        quote_map,
        base_map,
    })
}

