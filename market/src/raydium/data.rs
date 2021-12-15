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

    authority: String,
    #[serde(rename = "openOrders")]
    open_orders: String,
    #[serde(rename = "targetOrders")]
    target_orders: String,
    #[serde(rename = "baseVault")]
    base_vault: String,
    #[serde(rename = "quoteVault")]
    quote_vault: String,
    #[serde(rename = "marketProgramId")]
    market_program_id: String,
    #[serde(rename = "marketId")]
    market_id: String,
    #[serde(rename = "marketBids")]
    market_bids: String,
    #[serde(rename = "marketAsks")]
    market_asks: String,
    #[serde(rename = "marketEventQueue")]
    market_event_queue: String,
    #[serde(rename = "marketVaultSigner")]
    market_vault_signer: String,

}

pub fn load_data_from_file(quote_mint: &String, base_mint: &String) -> Result<MarketOptMap> {
    let market_main_path = "./raydium_pool.json".to_string();

    let raw_info = fs::read_to_string(market_main_path).expect("Error read file");
    let vec: Vec<RawMarketPool> = serde_json::from_str(&raw_info)?;

    let mut quote_map = HashMap::new();
    let mut base_map = HashMap::new();

    for pool in &vec {
        let mut data = HashMap::new();
        data.insert("poolMint".to_string(), pool.authority.clone());
        data.insert("openOrders".to_string(), pool.open_orders.clone());
        data.insert("targetOrders".to_string(), pool.target_orders.clone());
        data.insert("baseVault".to_string(), pool.base_vault.clone());
        data.insert("quoteVault".to_string(), pool.quote_vault.clone());
        data.insert("marketProgramId".to_string(), pool.market_program_id.clone());
        data.insert("marketId".to_string(), pool.market_id.clone());
        data.insert("marketBids".to_string(), pool.market_bids.clone());
        data.insert("marketAsks".to_string(), pool.market_asks.clone());
        data.insert("marketEventQueue".to_string(), pool.market_event_queue.clone());
        data.insert("marketVaultSigner".to_string(), pool.market_vault_signer.clone());

        if pool.quote_mint.eq(quote_mint) {
            let market_pool = MarketPool {
                pool_key: Pubkey::from_str(&pool.id).unwrap(),
                quote_mint_key: Pubkey::from_str(&pool.quote_mint).unwrap(),
                base_mint_key: Pubkey::from_str(&pool.base_mint).unwrap(),
                quote_value_key: Pubkey::from_str(&pool.market_quote_vault).unwrap(),
                base_value_key: Pubkey::from_str(&pool.market_base_vault).unwrap(),
                is_quote_to_base: true,
                data: data.clone(),
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
                data: data.clone(),
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
                data: data.clone(),
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
                data: data.clone(),
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

