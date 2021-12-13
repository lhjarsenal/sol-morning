use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use anyhow::Result;

use std::thread;
use std::sync::mpsc;

const SOLSCAN_TRANSACTION_URL: &str = "https://public-api.solscan.io/account/transactions?account=";
const SOLSCAN_DETAIL_URL: &str = "https://public-api.solscan.io/transaction/";

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryRequest {
    pub address: String,
    pub before: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TxResponse {
    #[serde(rename = "blockTime")]
    block_time: u64,
    slot: u64,
    #[serde(rename = "txHash")]
    tx_hash: String,
    fee: u32,
    status: String,
    lamport: u64,
    signer: Vec<String>,
    #[serde(rename = "parsedInstruction")]
    parsed_instruction: Vec<HashMap<String, String>>,
    detail: Option<TxDetail>,
}

impl TxResponse {
    pub fn set_detail(&mut self, detail: Option<TxDetail>) {
        self.detail =detail;
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TxDetail {
    block_time: u64,
    slot: u64,
    tx_hash: String,
    fee: u32,
    status: String,
    lamport: u64,
    signer: Vec<String>,
    log_message: Vec<String>,
    input_account: Vec<TxInputAccount>,
    recent_blockhash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TxInputAccount {
    account: String,
    signer: bool,
    writable: bool,
    pre_balance: u64,
    post_balance: u64,
}


impl HistoryRequest {
    pub fn get_history(&self) -> Vec<TxResponse> {
        let mut transaction_url: String = SOLSCAN_TRANSACTION_URL.to_owned() + &self.address.clone();

        match &self.before {
            Some(s) => {
                let before_hash = format!("&beforeHash={}", s);
                transaction_url.push_str(&*before_hash);
            }
            None => {}
        }
        let mut res = reqwest::blocking::get(transaction_url).unwrap();
        let mut tx_res = res.json::<Vec<TxResponse>>().unwrap();

        // for mut tx in &tx_res {
        //     thread::spawn(move || {
        //         let mut detail_url: String = SOLSCAN_DETAIL_URL.to_owned() + &tx.tx_hash.clone();
        //         let mut detail = reqwest::blocking::get(detail_url).unwrap();
        //
        //         // tm.send(detail.json::<TxDetail>().unwrap()).unwrap();
        //         let tx_detail = detail.json::<TxDetail>().unwrap();
        //         &tx.set_detail(Some(tx_detail));
        //     });
        // }

        tx_res
    }
}
