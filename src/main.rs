#![feature(proc_macro_hygiene)]
#![feature(decl_macro)]
#![feature(total_cmp)]

pub mod api;
pub mod response;
pub mod node_client;
mod opt_core;
mod rpc_client;

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
extern crate hyper;
extern crate reqwest;
extern crate safe_transmute;


use rocket_contrib::json::{Json, JsonValue};
use api::{ToDo, OptRequest, TokenAddr, RawTokenAddr};
use response::OptResponse;
use std::fs;
use anyhow::Result;
use rpc_client::{HistoryRequest, TxResponse};
use hyper::Client;
use rocket::http::Method;
use rocket_cors::{Cors, AllowedOrigins, AllowedHeaders};


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

#[get("/token_list")]
fn token_list() -> Json<Vec<RawTokenAddr>> {
    let token_main_path = "./token_mint.json".to_string();
    let raw_info = fs::read_to_string(token_main_path).expect("Error read file");
    let vec: Vec<RawTokenAddr> = serde_json::from_str(&raw_info).unwrap();
    Json(vec)
}

#[get("/pool_list")]
fn pool_list() -> Json<Vec<RawTokenAddr>> {
    let token_main_path = "./pool.json".to_string();
    let raw_info = fs::read_to_string(token_main_path).expect("Error read file");
    let vec: Vec<RawTokenAddr> = serde_json::from_str(&raw_info).unwrap();
    Json(vec)
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
        Err(e) => {
            response = Json(OptResponse {
                code: 101,
                msg: "error".to_string(),
                data: vec![],
            });
        }
    }
    response
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index, opt_swap, token_list, history])
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

