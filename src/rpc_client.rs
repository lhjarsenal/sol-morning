use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::thread;
use std::sync::mpsc;
use safe_transmute::alloc::sync::Arc;
use crate::pool::pool;
use pool::load_farm_data_from_file;

const SOLSCAN_TRANSACTION_URL: &str = "https://public-api.solscan.io/account/transactions?account=";
const SOLSCAN_DETAIL_URL: &str = "https://public-api.solscan.io/transaction/";
const SOLSCAN_ASSET_URL: &str = "https://api.solscan.io/account/tokens?price=1&address=";

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountRequest {
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OutApiResponse<T> {
    #[serde(rename = "succcess")]
    pub(crate) success: bool,
    pub(crate) data: Vec<T>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssetResponse {
    #[serde(rename = "tokenAddress")]
    token_address: String,
    #[serde(rename = "tokenAmount")]
    token_amount: TokenAmount,
    #[serde(rename = "tokenAccount")]
    token_account: String,
    #[serde(rename = "tokenName")]
    token_name: String,
    #[serde(rename = "tokenIcon")]
    token_icon: String,
    #[serde(rename = "rentEpoch")]
    rent_epoch: u64,
    lamports: u64,
    #[serde(rename = "tokenSymbol")]
    token_symbol: Option<String>,
    #[serde(rename = "priceUsdt")]
    price_usdt: Option<f64>,

}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenAmount {
    amount: String,
    decimals: u64,
    #[serde(rename = "uiAmount")]
    ui_amount: f64,
    #[serde(rename = "uiAmountString")]
    ui_amount_string: String,
}

impl AccountRequest {
    pub fn get_history(&self) -> Vec<TxResponse> {
        let mut transaction_url: String = SOLSCAN_TRANSACTION_URL.to_owned() + &self.address.clone();

        match &self.before {
            Some(s) => {
                let before_hash = format!("&beforeHash={}", s);
                transaction_url.push_str(&*before_hash);
            }
            None => {}
        }
        let res = reqwest::blocking::get(transaction_url).unwrap();
        let mut tx_res = res.json::<Vec<TxResponse>>().unwrap();

        let (tx, rx) = mpsc::channel();

        let arc_tx_res = Arc::new(tx_res.clone());

        for i in 0..arc_tx_res.len() {
            let tx_hash = arc_tx_res[i].tx_hash.clone();
            let tx = tx.clone();
            thread::spawn(move || {
                let detail_url: String = SOLSCAN_DETAIL_URL.to_owned() + &*tx_hash;
                let detail = reqwest::blocking::get(detail_url).unwrap();
                let tx_detail = detail.json::<TxDetail>().unwrap();
                let _send_result = tx.send((i, tx_detail));
            });
        }

        for i in rx.recv() {
            tx_res[i.0].set_detail(Option::from(i.1));
        }

        tx_res
    }

    pub fn get_assets(&self) -> Vec<AssetResponse> {
        let asset_url: String = SOLSCAN_ASSET_URL.to_owned() + &self.address.clone();

        let orca_farm_key = "Aquafarm".trim();

        let res = reqwest::blocking::get(asset_url).unwrap();
        let asset_res = res.json::<OutApiResponse<AssetResponse>>().unwrap();

        let asset_map: HashMap<String, AssetResponse> = asset_res.data
            .iter()
            .map(|x| {
                let key = x.token_address.clone();
                (key, x.clone())
            })
            .collect();

        //加载farm-pool对应关系
        let farm_main_path = "./resource/farm/orca.json".to_string();
        let farm_pool_map = load_farm_data_from_file(&farm_main_path).unwrap();

        let mut res = vec![];

        for asset in &mut asset_res.data.iter() {
            if asset.token_name.eq("") {
                continue;
            }
            if asset.token_name.contains(orca_farm_key) {
                let pool_address = farm_pool_map.get(&asset.token_address);
                if pool_address.is_some() {
                    let pool_option = asset_map.get(pool_address.unwrap());
                    if pool_option.is_some() {
                        let mut pool_asset = pool_option.unwrap().clone();
                        pool_asset.token_name = asset.token_name.clone();
                        pool_asset.token_icon = asset.token_icon.clone();
                        pool_asset.token_symbol = asset.token_symbol.clone();
                        res.push(pool_asset);
                    }
                }
            } else {
                res.push(asset.clone());
            }
        }

        res
    }
}
