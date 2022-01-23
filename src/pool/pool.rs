use crate::{api, PoolListResponse};
use crate::opt_core;
use crate::node_client;
use market::pool::{PoolInfo, PoolResponse, RawPool, TokenInfo};
use market::{orca, raydium};
use serde::{Serialize, Deserialize};
use solana_program::pubkey::Pubkey;
use solana_sdk::{commitment_config::CommitmentConfig, account::Account};
use solana_client::rpc_client::RpcClient;
use node_client::NetworkType;
use market::market::MarketType::*;
use anyhow::Result;
use market::raydium::stats::AmmInfo;
use opt_core::convert_to_info;
use api::load_token_data_from_file;
use spl_token_swap::processor::Processor;
use api::TokenAddr;
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use bytemuck::__core::ops::{Add, Mul, Div};
use std::collections::HashMap;
use std::fs;
use rocket_contrib::json::Json;
use solana_program::program_pack::Pack;
use spl_token::state::Mint;
use spl_token_swap::curve::base::SwapCurve;
use spl_token_swap::curve::calculator::{CurveCalculator, TradeDirection};
use spl_token_swap::curve::stable::StableCurve;
use spl_token_swap::state::SwapV1;
use crate::opt_core::get_swap_fee_ratio;

#[derive(Debug, Serialize, Deserialize)]
pub struct PoolRequest {
    pub market: Option<String>,
    pub token_mint_a: Option<String>,
    pub token_mint_b: Option<String>,
    pub lp_mint: Option<String>,
    pub farm_mint: Option<String>,
    pub slippage: Option<f32>,
    pub need_rate: Option<bool>,
}

impl PoolRequest {
    pub fn load_data(&self) -> Vec<PoolInfo> {

        //参数校验
        if self.lp_mint.is_none() {
            if self.token_mint_a.is_none() || self.token_mint_b.is_none() {
                return vec![];
            }
        }
        let mut need_raydium = false;
        let mut need_orca = false;
        match &self.market {
            Some(a) => {
                if a.eq("raydium") {
                    need_raydium = true;
                } else if a.eq("orca") {
                    need_orca = true;
                } else {
                    return vec![];
                }
            }
            None => {
                need_raydium = true;
                need_orca = true;
            }
        }


        let mut market_pool = vec![];

        if need_raydium {
            let raydium_pool = raydium::data::load_pool_from_file(self.lp_mint.clone(), self.token_mint_a.clone(),
                                                                  self.token_mint_b.clone());
            if raydium_pool.is_some() {
                market_pool.push(raydium_pool.unwrap());
            }
        }

        if need_orca {
            let orca_pool = orca::data::load_pool_from_file(self.lp_mint.clone(), self.token_mint_a.clone(),
                                                            self.token_mint_b.clone());
            if orca_pool.is_some() {
                market_pool.push(orca_pool.unwrap());
            }
        }

        market_pool
    }
}

pub fn cal_rate(pools: &[PoolInfo], slippage: &Option<f32>) -> Vec<PoolResponse> {

    //查询
    let token_main_path = "./token_mint.json".to_string();
    let tokens_adr = load_token_data_from_file(&token_main_path).expect("load token data fail");

    let mut keys: Vec<Pubkey> = vec![];
    for pool in pools {
        keys.push(pool.pool_key.clone());
        keys.push(pool.quote_value_key.clone());
        keys.push(pool.base_value_key.clone());
        keys.push(pool.lp_mint_key.clone());
    }

    let client = RpcClient::new(NetworkType::Mainnet.url().to_string());
    let commitment_config = CommitmentConfig::processed();
    let accounts: Vec<Option<Account>> = client.get_multiple_accounts_with_commitment(&keys, commitment_config).unwrap().value;

    let mut account_map = HashMap::new();
    for (index, value) in accounts.iter().enumerate() {
        let pubkey = &keys[index];
        match value {
            Some(account) => {
                account_map.insert(pubkey.to_string(), account.clone());
            }
            None => {}
        }
    }

    let mut res = vec![];

    for pool in pools {
        match &pool.market_type {
            Raydium(_x, _y) => {
                let mut pool_info = cal_raydium(&account_map, &tokens_adr, &pool).unwrap();
                match slippage {
                    Some(a) => {
                        let rate_fix = pool_info.rate.unwrap() / (1.0 + (a.clone() / 100.0));
                        pool_info.rate = Some(rate_fix);
                    }
                    None => {}
                }
                res.push(pool_info);
            }
            Orca(_x, _y) => {
                let pool_info = cal_orca(&account_map, &tokens_adr, &pool).unwrap();
                // match slippage {
                //     Some(a) => {
                //         let rate_fix = pool_info.rate.unwrap() / (1.0 + (a.clone() / 100.0));
                //         pool_info.rate = Some(rate_fix);
                //     }
                //     None => {}
                // }
                res.push(pool_info);
            }
            Saber(_x, _y) => {}
            Swap(_x, _y) => {}
            Serum(_x, _y) => {}
        }
    }

    res
}

pub fn load_pool_data(market: Option<String>) -> Vec<RawPool> {
    RawPool::load_all_pool_data(market)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RawFarm {
    #[serde(rename = "baseTokenMint")]
    pub lp_mint: String,
    #[serde(rename = "farmTokenMint")]
    pub farm_mint: String,
}

pub fn load_farm_data_from_file(path: &String) -> Result<HashMap<String, String>> {
    let raw_info = fs::read_to_string(path).expect("Error read file");
    let vec: Vec<RawFarm> = serde_json::from_str(&raw_info)?;
    let res: HashMap<String, String> = vec
        .iter()
        .map(|x| {
            (x.farm_mint.clone(), x.lp_mint.clone())
        })
        .collect();
    Ok(res)
}

pub fn load_pool_farm_data_from_file(path: &String) -> Result<HashMap<String, String>> {
    let raw_info = fs::read_to_string(path).expect("Error read file");
    let vec: Vec<RawFarm> = serde_json::from_str(&raw_info)?;
    let res: HashMap<String, String> = vec
        .iter()
        .map(|x| {
            (x.lp_mint.clone(), x.farm_mint.clone())
        })
        .collect();
    Ok(res)
}

pub fn fill_token_info(token_map: &HashMap<String, TokenAddr>, token_address: &str) -> Option<TokenInfo> {
    let token = token_map.get(token_address);
    let info = match token {
        Some(a) => {
            Some(TokenInfo {
                symbol: a.name.to_string(),
                address: a.mint.to_string(),
                decimals: a.decimal.clone(),
                name: a.description.to_string(),
                icon_uri: a.icon_uri.to_string(),
            })
        }
        None => {
            None
        }
    };
    info
}

fn cal_raydium(account_map: &HashMap<String, Account>,
               token_map: &HashMap<String, TokenAddr>,
               pool: &PoolInfo) -> Result<PoolResponse> {
    let amount_in = 1.0;

    let pool_ac = account_map.get(&pool.pool_key.to_string()).unwrap();
    let mut pool_clone = pool_ac.clone();
    let pool_ac_info = convert_to_info(&pool.pool_key, &mut pool_clone);
    let pool_info = AmmInfo::load_amm_mut(&pool_ac_info, false).unwrap();

    let quote_ac = account_map.get(&pool.quote_value_key.to_string()).unwrap();
    let mut quote_clone = quote_ac.clone();
    let quote_ac_info = convert_to_info(&pool.quote_value_key, &mut quote_clone);
    let quote_info = Processor::unpack_token_account(&quote_ac_info, &quote_ac.owner).unwrap();

    let base_ac = account_map.get(&pool.base_value_key.to_string()).unwrap();
    let mut base_clone = base_ac.clone();
    let base_ac_info = convert_to_info(&pool.base_value_key, &mut base_clone);
    let base_info = Processor::unpack_token_account(&base_ac_info, &base_ac.owner).unwrap();

    let basic: i128 = 10;
    let quote_token = token_map.get(&pool.quote_mint_key.to_string()).unwrap();
    let base_token = token_map.get(&pool.base_mint_key.to_string()).unwrap();
    let quote_pow = basic.pow(quote_token.decimal as u32);
    let base_pow = basic.pow(base_token.decimal as u32);
    let quote_amount = Decimal::from(quote_info.amount);
    let base_amount = Decimal::from(base_info.amount);

    let from_amount = Decimal::from_f64(amount_in * (base_pow as f64)).unwrap();
    let from_amount_with_fee = from_amount.mul(Decimal::from(pool_info.fees.swap_fee_denominator - pool_info.fees.swap_fee_numerator)).div(Decimal::from(pool_info.fees.swap_fee_denominator));
    let denominator = base_amount.add(from_amount_with_fee);
    let amount_out = quote_amount.mul(from_amount_with_fee).div(denominator);
    let mut amount_out_format = amount_out.div(Decimal::from(quote_pow));
    amount_out_format.rescale(quote_token.decimal as u32);

    let matket_type = pool.market_type.get_name();
    Ok(PoolResponse {
        market: matket_type.0,
        program_id: matket_type.1,
        pool_account: pool.pool_key.to_string(),
        quote_mint: pool.quote_mint_key.to_string(),
        base_mint: pool.base_mint_key.to_string(),
        lp_mint: pool.lp_mint_key.to_string(),
        quote_value: pool.quote_value_key.to_string(),
        base_value: pool.base_value_key.to_string(),
        rate: Some(amount_out_format.to_f32().unwrap()),
        data: pool.data.clone(),
    })
}

fn cal_orca(account_map: &HashMap<String, Account>,
            token_map: &HashMap<String, TokenAddr>,
            pool: &PoolInfo) -> Result<PoolResponse> {
    let amount_in = 1.0;

    let pool_ac = account_map.get(&pool.pool_key.to_string()).unwrap();
    let pool_clone = pool_ac.clone();
    let pool_info = SwapV1::unpack_from_slice(&pool_clone.data).unwrap();

    let quote_ac = account_map.get(&pool.quote_value_key.to_string()).unwrap();
    let mut quote_clone = quote_ac.clone();
    let quote_ac_info = convert_to_info(&pool.quote_value_key, &mut quote_clone);
    let quote_info = Processor::unpack_token_account(&quote_ac_info, &quote_ac.owner).unwrap();

    let base_ac = account_map.get(&pool.base_value_key.to_string()).unwrap();
    let mut base_clone = base_ac.clone();
    let base_ac_info = convert_to_info(&pool.base_value_key, &mut base_clone);
    let base_info = Processor::unpack_token_account(&base_ac_info, &base_ac.owner).unwrap();

    let pl_ac = account_map.get(&pool.lp_mint_key.to_string()).unwrap();
    let pl_info = Mint::unpack(&pl_ac.data).unwrap();

    let basic: i128 = 10;
    let quote_token = token_map.get(&pool.quote_mint_key.to_string()).unwrap();
    let base_token = token_map.get(&pool.base_mint_key.to_string()).unwrap();
    let quote_pow = basic.pow(quote_token.decimal as u32);
    let base_pow = basic.pow(base_token.decimal as u32);

    let fee_ratio = get_swap_fee_ratio(pool_info.fees.trade_fee_numerator,
                                       pool_info.fees.trade_fee_denominator,
                                       pool_info.fees.owner_trade_fee_numerator,
                                       pool_info.fees.owner_trade_fee_denominator);

    let from_amount = amount_in * (base_pow as f64);
    let from_amount_with_fee = from_amount - (from_amount * (fee_ratio as f64));
    let amount_out;

    let amp = pool.data.get("amp");
    if amp.is_some() {
        let amp_u64 = amp.unwrap().parse::<u64>().unwrap();
        let sc = StableCurve {
            amp: amp_u64
        };
        let sc_result = sc.swap_without_fees(from_amount_with_fee.to_u128().unwrap(),
                                             base_info.amount as u128,
                                             quote_info.amount as u128,
                                             TradeDirection::BtoA).unwrap();
        amount_out = Decimal::from_u128(sc_result.destination_amount_swapped).unwrap();
    } else {
        let sc = SwapCurve::default();
        let sc_result = sc.calculator.swap_without_fees(from_amount_with_fee.to_u128().unwrap(),
                                                        base_info.amount as u128,
                                                        quote_info.amount as u128,
                                                        TradeDirection::BtoA).unwrap();
        amount_out = Decimal::from_u128(sc_result.destination_amount_swapped).unwrap();
    }

    let mut amount_out_format = amount_out.div(Decimal::from(quote_pow));
    amount_out_format.rescale(quote_token.decimal as u32);

    let matket_type = pool.market_type.get_name();

    let mut pool_data = pool.data.clone();
    pool_data.insert("poolSupply".to_string(), pl_info.supply.to_string());
    pool_data.insert("quoteAmount".to_string(), quote_info.amount.to_string());
    pool_data.insert("baseAmount".to_string(), base_info.amount.to_string());

    // let pool_token_amount = orca::data::calculate_pool_deposit_amount(quote_info.amount, base_info.amount, pl_info.supply);
    // pool_data.insert("poolTokenAmount".to_string(), pool_token_amount.to_string());
    Ok(PoolResponse {
        market: matket_type.0,
        program_id: matket_type.1,
        pool_account: pool.pool_key.to_string(),
        quote_mint: pool.quote_mint_key.to_string(),
        base_mint: pool.base_mint_key.to_string(),
        lp_mint: pool.lp_mint_key.to_string(),
        quote_value: pool.quote_value_key.to_string(),
        base_value: pool.base_value_key.to_string(),
        rate: Some(amount_out_format.to_f32().unwrap()),
        data: pool_data,
    })
}


pub fn pool_list(page: Option<u32>, pagesize: Option<u32>,
             lp_mint: Option<String>, farm_mint: Option<String>,
             address: Option<String>, market: Option<String>,
             search: Option<String>) -> Json<PoolListResponse> {
    let mut vec: Vec<RawPool> = load_pool_data(market);
    //查询
    let token_main_path = "./token_mint.json".to_string();
    let tokens_adr = api::load_token_data_from_file(&token_main_path).expect("load token data fail");

    for mut pool in &mut vec {
        pool.quote_token = fill_token_info(&tokens_adr, &pool.quote_mint);
        pool.base_token = fill_token_info(&tokens_adr, &pool.base_mint);
    }

    //查询固定某一个lp_mint
    match lp_mint {
        Some(a) => {
            for pool in &mut vec {
                if pool.lp_mint.eq(&a) {
                    return Json(PoolListResponse {
                        total: 1,
                        pagesize: 1,
                        page: 1,
                        data: vec![pool.clone()],
                    });
                }
            }
            return Json(PoolListResponse {
                total: 0,
                pagesize: 1,
                page: 1,
                data: vec![],
            });
        }
        None => {}
    }

    //查询固定某一个farm_mint
    match farm_mint {
        Some(a) => {

            //加载farm-pool对应关系
            let farm_main_path = "./resource/farm/orca.json".to_string();
            let farm_pool_map = load_farm_data_from_file(&farm_main_path).unwrap();
            let lp_mint = farm_pool_map.get(&a);
            if lp_mint.is_some() {
                for pool in &mut vec {
                    if pool.lp_mint.eq(lp_mint.unwrap()) {
                        return Json(PoolListResponse {
                            total: 1,
                            pagesize: 1,
                            page: 1,
                            data: vec![pool.clone()],
                        });
                    }
                }
            } else {
                return Json(PoolListResponse {
                    total: 0,
                    pagesize: 1,
                    page: 1,
                    data: vec![],
                });
            }

            return Json(PoolListResponse {
                total: 0,
                pagesize: 1,
                page: 1,
                data: vec![],
            });
        }
        None => {}
    }

    //固定查询某个token
    match address {
        Some(add) => {
            let match_add = add.trim();
            vec = vec
                .into_iter()
                .filter(|x|
                    x.quote_mint.eq(match_add) || x.base_mint.eq(match_add)
                ).collect();
        }
        None => {}
    }

    //模糊查询symbol
    match search {
        Some(symbol) => {
            let match_symbol = symbol.trim();
            vec = vec
                .into_iter()
                .filter(|x|
                    (x.quote_token.is_some() && x.quote_token.as_ref().unwrap().symbol.to_uppercase().trim().contains(match_symbol)) ||
                        (x.base_token.is_some() && x.base_token.as_ref().unwrap().symbol.clone().to_uppercase().trim().contains(match_symbol))
                ).collect();
        }
        None => {}
    }
    let start_page;
    let mut size = 50;
    let start_index;
    let total = vec.len() as u32;

    if total == 0 {
        return Json(PoolListResponse {
            total,
            pagesize: total,
            page: 1,
            data: vec![],
        });
    }

    match page {
        Some(p) => {
            start_page = p - 1;
        }
        None => {
            return Json(PoolListResponse {
                total,
                pagesize: total,
                page: 1,
                data: vec,
            });
        }
    }

    match pagesize {
        Some(s) => {
            size = s;
        }
        None => {}
    }

    start_index = start_page * size;
    let mut end_index = start_index + size;

    if end_index >= total {
        end_index = total;
    }

    let res = vec[start_index as usize..end_index as usize].to_vec();

    Json(PoolListResponse {
        total,
        pagesize: size,
        page: start_page + 1,
        data: res,
    })
}

pub fn pool_info(req: Json<PoolRequest>) -> Json<Vec<PoolResponse>> {
    let mut request = req.0;

    if request.farm_mint.is_some() {
        //加载farm-pool对应关系
        let farm_mint = request.farm_mint.clone().unwrap();
        let farm_main_path = "./resource/farm/orca.json".to_string();
        let farm_pool_map = load_farm_data_from_file(&farm_main_path).unwrap();
        let lp_mint = farm_pool_map.get(&farm_mint);
        if lp_mint.is_some() {
            request.lp_mint = lp_mint.cloned();
        } else {
            return Json(vec![]);
        }
    }

    let opt_pool = request.load_data();

    let pool_info = match request.need_rate {
        Some(bool) => {
            if bool {
                cal_rate(&opt_pool, &request.slippage)
            } else {
                opt_pool.iter()
                    .map(|x| -> PoolResponse{
                        let market_type = x.market_type.get_name();
                        PoolResponse {
                            market: market_type.0,
                            program_id: market_type.1,
                            pool_account: x.pool_key.to_string(),
                            quote_mint: x.quote_mint_key.to_string(),
                            base_mint: x.base_mint_key.to_string(),
                            lp_mint: x.lp_mint_key.to_string(),
                            quote_value: x.quote_value_key.to_string(),
                            base_value: x.base_value_key.to_string(),
                            rate: None,
                            data: x.data.clone(),
                        }
                    }).collect()
            }
        }
        None => {
            opt_pool.iter()
                .map(|x| -> PoolResponse{
                    let market_type = x.market_type.get_name();
                    PoolResponse {
                        market: market_type.0,
                        program_id: market_type.1,
                        pool_account: x.pool_key.to_string(),
                        quote_mint: x.quote_mint_key.to_string(),
                        base_mint: x.base_mint_key.to_string(),
                        lp_mint: x.lp_mint_key.to_string(),
                        quote_value: x.quote_value_key.to_string(),
                        base_value: x.base_value_key.to_string(),
                        rate: None,
                        data: x.data.clone(),
                    }
                }).collect()
        }
    };

    Json(pool_info)
}