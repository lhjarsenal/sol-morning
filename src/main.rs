#![feature(proc_macro_hygiene)]
#![feature(decl_macro)]
#![feature(total_cmp)]

pub mod api;
pub mod response;
pub mod node_client;
mod opt_core;
mod rpc_client;
pub mod pool;
pub mod token;

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
use api::OptRequest;
use response::{OptResponse, TokenListResponse};
use rpc_client::{AccountRequest, TxResponse};
use rocket::http::Method;
use rocket_cors::{Cors, AllowedOrigins, AllowedHeaders};
use pool::pool::PoolRequest;
use market::pool::PoolResponse;
use crate::response::PoolListResponse;
use crate::rpc_client::{AssetResponse, OutApiResponse};
use crate::token::token::WoreholeAddress;


#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/history?<address>&<before>")]
fn history(address: String, before: Option<String>) -> Json<Vec<TxResponse>> {
    let req = AccountRequest {
        address,
        before,
    };
    Json(req.get_history())
}

#[get("/assets?<address>")]
fn assets(address: String) -> Json<OutApiResponse<AssetResponse>> {
    let req = AccountRequest {
        address,
        before: None,
    };

    Json(OutApiResponse {
        success: true,
        data: req.get_assets(),
    })
}

#[get("/token_list?<page>&<pagesize>&<search>&<address>&<symbol>&<chain>")]
fn token_list(page: Option<u32>, pagesize: Option<u32>,
              search: Option<String>, address: Option<String>,
              symbol: Option<String>, chain:Option<String>) -> Json<TokenListResponse> {
    match chain {
        None=>{
            token::token::token_list(page, pagesize, search, address, symbol)
        },
        Some(chain_type)=>{
            if chain_type.eq("ethereum"){
                token::token::eth_tokens(page, pagesize, search, address, symbol)
            }else {
                Json(TokenListResponse{
                    total: 0,
                    pagesize: 0,
                    page: 0,
                    data: vec![]
                })
            }

        }
    }

}

#[get("/pool_list?<page>&<pagesize>&<lp_mint>&<farm_mint>&<address>&<market>&<search>")]
fn pool_list(page: Option<u32>, pagesize: Option<u32>,
             lp_mint: Option<String>, farm_mint: Option<String>,
             address: Option<String>, market: Option<String>,
             search: Option<String>) -> Json<PoolListResponse> {
    pool::pool::pool_list(page, pagesize, lp_mint, farm_mint, address, market, search)
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
    pool::pool::pool_info(req)
}

// #[get("/eth_tokens?<page>&<pagesize>&<search>&<address>&<symbol>")]
// fn eth_tokens(page: Option<u32>, pagesize: Option<u32>,
//               search: Option<String>, address: Option<String>,
//               symbol: Option<String>) -> Json<TokenListResponse> {
//     token::token::eth_tokens(page, pagesize, search, address, symbol)
// }

#[get("/bridge_token?<source_chain>&<to_chain>&<origin_address>&<wrap_address>")]
fn bridge_token(source_chain: String, to_chain: String,
                origin_address: Option<String>, wrap_address: Option<String>) -> Json<WoreholeAddress> {

    if origin_address.is_some(){
        Json(token::token::bridge_token_by_origin(source_chain, to_chain, origin_address.unwrap()))
    }else if wrap_address.is_some(){
        Json(token::token::bridge_token_by_wrap(source_chain, to_chain, wrap_address.unwrap()))
    }else {
        Json(WoreholeAddress{
            origin_address: "".to_string(),
            target_token: None
        })
    }
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index, assets, opt_swap, token_list,
            pool_list, history, pool_info, bridge_token])
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

