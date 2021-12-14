use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use anyhow::Result;

use std::thread;
use std::sync::mpsc;
use safe_transmute::alloc::sync::Arc;

const SOLSCAN_TRANSACTION_URL: &str = "https://public-api.solscan.io/account/transactions?account=";
const SOLSCAN_DETAIL_URL: &str = "https://public-api.solscan.io/transaction/";

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryRequest {
    pub address: String,
    pub before: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
        self.detail = detail;
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TxDetail {
    #[serde(rename = "blockTime")]
    block_time: u64,
    slot: u64,
    #[serde(rename = "txHash")]
    tx_hash: String,
    fee: u32,
    status: String,
    lamport: u64,
    signer: Vec<String>,
    #[serde(rename = "logMessage")]
    log_message: Vec<String>,
    #[serde(rename = "inputAccount")]
    input_account: Vec<TxInputAccount>,
    #[serde(rename = "recentBlockhash")]
    recent_blockhash: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TxInputAccount {
    account: String,
    signer: bool,
    writable: bool,
    #[serde(rename = "preBalance")]
    pre_balance: u64,
    #[serde(rename = "postBalance")]
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

        let (tx, rx) = mpsc::channel();

        let arc_tx_res = Arc::new(tx_res.clone());

        for i in 0..arc_tx_res.len() {
            let tx_hash = arc_tx_res[i].tx_hash.clone();
            let tx = tx.clone();
            thread::spawn(move || {
                let mut detail_url: String = SOLSCAN_DETAIL_URL.to_owned() + &*tx_hash;
                let mut detail = reqwest::blocking::get(detail_url).unwrap();
                let tx_detail = detail.json::<TxDetail>().unwrap();
                tx.send((i, tx_detail));
            });
        }

        for i in rx.recv() {
            tx_res[i.0].set_detail(Option::from(i.1));
        }

        tx_res
    }
}
