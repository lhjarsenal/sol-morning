use crate::market;
use serde::{Serialize, Deserialize};
use market::market::{MarketSwap, MarketType};
use std::collections::HashMap;
use solana_sdk::account::Account;
use market::market::MarketType::*;


#[derive(Debug, Serialize, Deserialize)]
pub struct OptInitData {
    pub account_map: HashMap<String, Account>,
    pub swaps: Vec<MarketSwap>,
}

impl OptInitData {
    fn calculate(&self) {
        for swap in self.swaps.iter() {
            let market_type = swap.market_type.clone();
            match market_type {
                Raydium(x, y) => {}
                Orca(x, y) => {}
                Saber(x, y) => {}
                Swap(x, y) => {}
                Serum(x, y) => {}
            }
        }
    }
}