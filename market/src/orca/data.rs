use crate::market;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use std::fs;
use std::collections::HashMap;
use solana_program::pubkey::Pubkey;
use rust_decimal::prelude::FromStr;
use market::{MarketPool, MarketOptMap, MarketType};
use crate::pool::PoolInfo;

const ORCA_MARKET: &str = "orca";
const ORCA_PROGRAM_ID: &str = "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP";

#[derive(Serialize, Deserialize, Debug)]
pub struct RawMarketPool {
    pub account: String,
    pub quote: RawMarketToken,
    pub base: RawMarketToken,

    pub authority: String,
    pub pool_mint: String,
    pub fee_account: String,
    pub amp: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RawMarketToken {
    pub mint: String,
    pub reserves: String,
}

pub fn load_data_from_file(quote_mint: &String, base_mint: &String) -> Result<MarketOptMap> {
    let market_main_path = "./orca_pool.json".to_string();

    let raw_info = fs::read_to_string(market_main_path).expect("Error read file");
    let vec: Vec<RawMarketPool> = serde_json::from_str(&raw_info)?;

    let mut quote_map = HashMap::new();
    let mut base_map = HashMap::new();

    for pool in &vec {
        let mut data = HashMap::new();
        data.insert("authority".to_string(), pool.authority.clone());
        data.insert("poolMint".to_string(), pool.pool_mint.clone());
        data.insert("feeAccount".to_string(), pool.fee_account.clone());
        if pool.quote.mint.eq(quote_mint) {
            data.insert("poolQuoteValue".to_string(), pool.quote.reserves.clone());
            data.insert("poolBaseValue".to_string(), pool.base.reserves.clone());
            let market_pool = MarketPool {
                pool_key: Pubkey::from_str(&pool.account).unwrap(),
                quote_mint_key: Pubkey::from_str(&pool.quote.mint).unwrap(),
                base_mint_key: Pubkey::from_str(&pool.base.mint).unwrap(),
                quote_value_key: Pubkey::from_str(&pool.quote.reserves).unwrap(),
                base_value_key: Pubkey::from_str(&pool.base.reserves).unwrap(),
                is_quote_to_base: true,
                amp: pool.amp,
                data: data.clone(),
            };
            quote_map.insert(pool.base.mint.clone(), market_pool);
        }
        if pool.base.mint.eq(quote_mint) {
            data.insert("poolQuoteValue".to_string(), pool.base.reserves.clone());
            data.insert("poolBaseValue".to_string(), pool.quote.reserves.clone());
            let market_pool = MarketPool {
                pool_key: Pubkey::from_str(&pool.account).unwrap(),
                quote_mint_key: Pubkey::from_str(&pool.quote.mint).unwrap(),
                base_mint_key: Pubkey::from_str(&pool.base.mint).unwrap(),
                quote_value_key: Pubkey::from_str(&pool.quote.reserves).unwrap(),
                base_value_key: Pubkey::from_str(&pool.base.reserves).unwrap(),
                is_quote_to_base: false,
                amp: pool.amp,
                data: data.clone(),
            };
            quote_map.insert(pool.quote.mint.clone(), market_pool);
        }
        if pool.quote.mint.eq(base_mint) {
            data.insert("poolQuoteValue".to_string(), pool.base.reserves.clone());
            data.insert("poolBaseValue".to_string(), pool.quote.reserves.clone());
            let market_pool = MarketPool {
                pool_key: Pubkey::from_str(&pool.account).unwrap(),
                quote_mint_key: Pubkey::from_str(&pool.quote.mint).unwrap(),
                base_mint_key: Pubkey::from_str(&pool.base.mint).unwrap(),
                quote_value_key: Pubkey::from_str(&pool.quote.reserves).unwrap(),
                base_value_key: Pubkey::from_str(&pool.base.reserves).unwrap(),
                is_quote_to_base: false,
                amp: pool.amp,
                data: data.clone(),
            };
            base_map.insert(pool.base.mint.clone(), market_pool);
        }
        if pool.base.mint.eq(base_mint) {
            data.insert("poolQuoteValue".to_string(), pool.quote.reserves.clone());
            data.insert("poolBaseValue".to_string(), pool.base.reserves.clone());
            let market_pool = MarketPool {
                pool_key: Pubkey::from_str(&pool.account).unwrap(),
                quote_mint_key: Pubkey::from_str(&pool.quote.mint).unwrap(),
                base_mint_key: Pubkey::from_str(&pool.base.mint).unwrap(),
                quote_value_key: Pubkey::from_str(&pool.quote.reserves).unwrap(),
                base_value_key: Pubkey::from_str(&pool.base.reserves).unwrap(),
                is_quote_to_base: true,
                amp: pool.amp,
                data: data.clone(),
            };
            base_map.insert(pool.quote.mint.clone(), market_pool);
        }
    }

    Ok(MarketOptMap {
        market_type: MarketType::Orca(ORCA_MARKET.to_string().clone(), ORCA_PROGRAM_ID.to_string().clone()),
        quote_mint: quote_mint.clone(),
        base_mint: base_mint.clone(),
        quote_map,
        base_map,
    })
}

pub fn load_pool_from_file(lp_mint: Option<String>,
                           quote_mint: Option<String>,
                           base_mint: Option<String>) -> Option<PoolInfo> {
    let pool_main_path = "./resource/pool/orca.json".to_string();

    let raw_info = fs::read_to_string(pool_main_path).expect("Error read file");
    let vec: Vec<RawMarketPool> = serde_json::from_str(&raw_info).ok()?;

    let pool_info = match lp_mint {
        Some(lp) => {
            //通过pool地址筛选
            for raw_pool in vec {
                if raw_pool.pool_mint.eq(&lp) {
                    let mut data = HashMap::new();
                    if raw_pool.amp.is_some() {
                        data.insert("amp".to_string(), raw_pool.amp.unwrap().to_string());
                    }
                    data.insert("authority".to_string(), raw_pool.authority.clone());
                    data.insert("feeAccount".to_string(), raw_pool.fee_account.clone());
                    return Some(PoolInfo {
                        market_type: MarketType::Orca(ORCA_MARKET.to_string(), ORCA_PROGRAM_ID.to_string()),
                        pool_key: Pubkey::from_str(&raw_pool.account).unwrap(),
                        quote_mint_key: Pubkey::from_str(&raw_pool.quote.mint).unwrap(),
                        base_mint_key: Pubkey::from_str(&raw_pool.base.mint).unwrap(),
                        lp_mint_key: Pubkey::from_str(&raw_pool.pool_mint).unwrap(),
                        quote_value_key: Pubkey::from_str(&raw_pool.quote.reserves).unwrap(),
                        base_value_key: Pubkey::from_str(&raw_pool.base.reserves).unwrap(),
                        data,
                    });
                }
            }
            None
        }
        None => {
            //通过 quote/base token对mint地址筛选
            let quote_mint_address = quote_mint.unwrap().clone();
            let base_mint_address = base_mint.unwrap().clone();

            for raw_pool in vec {
                if quote_mint_address.eq(&raw_pool.quote.mint) && base_mint_address.eq(&raw_pool.base.mint) {
                    let mut data = HashMap::new();
                    if raw_pool.amp.is_some() {
                        data.insert("amp".to_string(), raw_pool.amp.unwrap().to_string());
                    }
                    data.insert("authority".to_string(), raw_pool.authority.clone());
                    data.insert("feeAccount".to_string(), raw_pool.fee_account.clone());
                    return Some(PoolInfo {
                        market_type: MarketType::Orca(ORCA_MARKET.to_string(), ORCA_PROGRAM_ID.to_string()),
                        pool_key: Pubkey::from_str(&raw_pool.account).unwrap(),
                        quote_mint_key: Pubkey::from_str(&raw_pool.quote.mint).unwrap(),
                        base_mint_key: Pubkey::from_str(&raw_pool.base.mint).unwrap(),
                        lp_mint_key: Pubkey::from_str(&raw_pool.pool_mint).unwrap(),
                        quote_value_key: Pubkey::from_str(&raw_pool.quote.reserves).unwrap(),
                        base_value_key: Pubkey::from_str(&raw_pool.base.reserves).unwrap(),
                        data,
                    });
                } else if quote_mint_address.eq(&raw_pool.base.mint) && base_mint_address.eq(&raw_pool.quote.mint) {
                    let mut data = HashMap::new();
                    if raw_pool.amp.is_some() {
                        data.insert("amp".to_string(), raw_pool.amp.unwrap().to_string());
                    }
                    data.insert("authority".to_string(), raw_pool.authority.clone());
                    data.insert("feeAccount".to_string(), raw_pool.fee_account.clone());
                    return Some(PoolInfo {
                        market_type: MarketType::Orca(ORCA_MARKET.to_string(), ORCA_PROGRAM_ID.to_string()),
                        pool_key: Pubkey::from_str(&raw_pool.account).unwrap(),
                        quote_mint_key: Pubkey::from_str(&raw_pool.quote.mint).unwrap(),
                        base_mint_key: Pubkey::from_str(&raw_pool.base.mint).unwrap(),
                        lp_mint_key: Pubkey::from_str(&raw_pool.pool_mint).unwrap(),
                        quote_value_key: Pubkey::from_str(&raw_pool.quote.reserves).unwrap(),
                        base_value_key: Pubkey::from_str(&raw_pool.base.reserves).unwrap(),
                        data,
                    });
                }
            }
            None
        }
    };
    pool_info
}

pub fn calculate_pool_deposit_amount(quote_reserves: u64, _base_reserves: u64, _pool_supply: u64) -> u64 {
    let _quote_amount = quote_reserves * (1 + 1000) / 1000;
    // let base_amount = base_reserves * (1 + 1000) / 1000;
    // quote_amount * 1000 * pool_supply / quote_reserves / (1 + 1000)
    0
}

