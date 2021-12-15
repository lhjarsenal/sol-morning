use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use solana_program::pubkey::Pubkey;
use anyhow::Result;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MarketType {
    Raydium(String, String),
    Orca(String, String),
    Saber(String, String),
    Swap(String, String),
    Serum(String, String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketOptMap {
    pub market_type: MarketType,
    pub quote_mint: String,
    pub base_mint: String,
    pub quote_map: HashMap<String, MarketPool>,
    pub base_map: HashMap<String, MarketPool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MarketPool {
    pub pool_key: Pubkey,
    pub quote_mint_key: Pubkey,
    pub base_mint_key: Pubkey,
    pub quote_value_key: Pubkey,
    pub base_value_key: Pubkey,
    pub is_quote_to_base: bool,
    pub data: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketSwap {
    pub market_type: MarketType,
    pub step: Vec<MarketPool>,
}

impl MarketOptMap {
    pub fn filer_swap(&self) -> Result<Vec<MarketSwap>> {
        let mut res = vec![];

        //先判断有没有直接swap的交易对
        let step = &self.base_map.get(&self.quote_mint);
        match step {
            Some(a) => {
                return Ok(vec![MarketSwap {
                    market_type: self.market_type.clone(),
                    step: vec![(*a).clone()],
                }]);
            }
            None => {}
        }

        for (key, value) in &self.quote_map {
            let source = &self.base_map.get(key);
            match source {
                Some(a) => {
                    res.push(MarketSwap {
                        market_type: self.market_type.clone(),
                        step: vec![value.clone(), (*a).clone()],
                    })
                }
                None => {}
            }
        }

        Ok(res)
    }
}

