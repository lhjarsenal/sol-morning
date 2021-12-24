use crate::market::MarketType;
use serde::{Serialize, Deserialize};
use solana_program::pubkey::Pubkey;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PoolInfo {
    pub market_type: MarketType,
    pub pool_key: Pubkey,
    pub quote_mint_key: Pubkey,
    pub base_mint_key: Pubkey,
    pub lp_mint_key: Pubkey,
    pub quote_value_key: Pubkey,
    pub base_value_key: Pubkey,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PoolResponse {
    pub market:String,
    pub program_id:String,
    pub pool_account: String,
    pub quote_mint: String,
    pub base_mint: String,
    pub lp_mint: String,
    pub quote_value: String,
    pub base_value: String,
    pub rate: Option<f64>,
}

impl PoolInfo {
    pub fn filer_pool(&self){

    }
}

