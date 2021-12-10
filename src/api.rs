use serde::{Serialize, Deserialize};
use anyhow::Result;
use std::fs;
use std::collections::HashMap;
use solana_program::pubkey::Pubkey;
use rust_decimal::prelude::FromStr;
use rocket_contrib::json::Json;

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
    pub exclude: Vec<String>,
}

impl OptRequest {
    pub fn loadData(&self) {

        //查询
        let token_main_path = "./token_mint.json".to_string();
        let tokens_adr = load_token_data_from_file(&token_main_path).expect("load data fail");

        let quote_token = tokens_adr.get(&self.quote_mint).expect("pubKey not found");
        let base_token = tokens_adr.get(&self.base_mint).expect("pubKey not found");
        println!("quote={},base={}", quote_token.mint.to_string(), base_token.mint.to_string());
        println!("json={:?}",Json(tokens_adr));
        let markets = vec!["raydium", "orca", "saber", "swap", "serum"];
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RawTokenAddr {
    pub name: String,
    pub mint: String,
    pub decimal: u8,
    pub description: String,
}

#[derive(Debug, Clone)]
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
            let key = x.mint.clone();
            (key, TokenAddr {
                name: (x.name).to_string(),
                mint: Pubkey::from_str(&x.mint).unwrap(),
                decimal: x.decimal,
                description: (x.description).to_string(),
            })
        })
        .collect();
    Ok(res)
}

