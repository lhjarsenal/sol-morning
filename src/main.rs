#![feature(proc_macro_hygiene)]
#![feature(decl_macro)]
#![feature(total_cmp)]

pub mod api;
pub mod response;
pub mod node_client;
mod opt_core;
mod rpc_client;
pub mod pool;

#[macro_use]
extern crate rocket;
extern crate rocket_contrib;
extern crate rocket_cors;
extern crate serde;
extern crate anyhow;
extern crate solana_program;
extern crate solana_sdk;

extern crate rust_decimal;
extern crate market;
extern crate solana_client;
extern crate bytemuck;
extern crate spl_token_swap;
extern crate num_traits;
extern crate reqwest;
extern crate safe_transmute;


use rocket_contrib::json::Json;
use api::{OptRequest, RawTokenAddr};
use response::{OptResponse, TokenListResponse};
use std::fs;
use rpc_client::{HistoryRequest, TxResponse};
use rocket::http::Method;
use rocket_cors::{Cors, AllowedOrigins, AllowedHeaders};
use pool::pool::{PoolRequest, cal_rate};
use market::pool::{PoolResponse, RawPool};
use pool::pool::load_pool_data;
use crate::response::PoolListResponse;


#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/history?<address>&<before>")]
fn history(address: String, before: Option<String>) -> Json<Vec<TxResponse>> {
    let req = HistoryRequest {
        address,
        before,
    };
    Json(req.get_history())
}

#[get("/token_list?<page>&<pagesize>&<search>&<address>")]
fn token_list(page: Option<u32>, pagesize: Option<u32>, search: Option<String>, address: Option<String>) -> Json<TokenListResponse> {
    let token_main_path = "./token_mint.json".to_string();
    let raw_info = fs::read_to_string(token_main_path).expect("Error read file");
    let mut vec: Vec<RawTokenAddr> = serde_json::from_str(&raw_info).unwrap();

    //查询固定某一个address
    match address {
        Some(a) => {
            for token in vec.iter() {
                if token.address.eq(&a) {
                    return Json(TokenListResponse {
                        total: 1,
                        pagesize: 1,
                        page: 1,
                        data: vec![token.clone()],
                    });
                }
            }
            return Json(TokenListResponse {
                total: 0,
                pagesize: 1,
                page: 1,
                data: vec![],
            });
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
                    x.symbol.to_uppercase().trim().contains(match_symbol)
                ).collect();
        }
        None => {}
    }

    let start_page;
    let mut size = 50;
    let start_index;
    let total = vec.len() as u32;
    match page {
        Some(p) => {
            start_page = p - 1;
        }
        None => {
            return Json(TokenListResponse {
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
        end_index = total - 1;
    }

    let res = vec[start_index as usize..end_index as usize].to_vec();
    Json(TokenListResponse {
        total,
        pagesize: size,
        page: start_page + 1,
        data: res,
    })
}

#[get("/pool_list?<page>&<pagesize>&<lp_mint>&<address>&<market>")]
fn pool_list(page: Option<u32>, pagesize: Option<u32>, lp_mint: Option<String>, address: Option<String>, market: Option<String>) -> Json<PoolListResponse> {
    let mut vec: Vec<RawPool> = load_pool_data(market);

    //查询固定某一个lp_mint
    match lp_mint {
        Some(a) => {
            for pool in &vec {
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

    //固定查询某个token
    match address {
        Some(symbol) => {
            let match_symbol = symbol.trim();
            vec = vec
                .into_iter()
                .filter(|x|
                    x.quote_mint.eq(match_symbol) || x.base_mint.eq(match_symbol)
                ).collect();
        }
        None => {}
    }

    let start_page;
    let mut size = 50;
    let start_index;
    let total = vec.len() as u32;
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
        end_index = total - 1;
    }

    let res = vec[start_index as usize..end_index as usize].to_vec();
    Json(PoolListResponse {
        total,
        pagesize: size,
        page: start_page + 1,
        data: res,
    })
}

#[post("/opt_swap", data = "<req>")]
fn opt_swap(req: Json<OptRequest>) -> Json<OptResponse> {
    println!("req={:?}", req.0);

    let mut opt_market = req.0.load_data();
    let opt_rank = opt_market.opt_best();
    let response;
    match opt_rank {
        Ok(res) => {
            response = Json(OptResponse {
                code: 0,
                msg: "success".to_string(),
                data: res,
            });
        }
        Err(_e) => {
            response = Json(OptResponse {
                code: 101,
                msg: "error".to_string(),
                data: vec![],
            });
        }
    }
    response
}

#[post("/pool_info", data = "<req>")]
fn pool_info(req: Json<PoolRequest>) -> Json<Vec<PoolResponse>> {
    println!("req={:?}", req.0);

    let opt_pool = req.0.load_data();

    let pool_info = match req.need_rate {
        Some(bool) => {
            if bool {
                cal_rate(&opt_pool)
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

fn main() {
    rocket::ignite()
        .mount("/", routes![index, opt_swap, token_list,pool_list, history, pool_info])
        .attach(get_cors())
        .launch();
}

fn get_cors() -> Cors {
    let allowed_origins = AllowedOrigins::All;
    rocket_cors::CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post, Method::Options].into_iter()
            .map(From::from).collect(),
        allowed_headers: AllowedHeaders::All,
        ..Default::default()
    }.to_cors().expect("cors config error")
}

