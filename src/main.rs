#![feature(proc_macro_hygiene)]
#![feature(decl_macro)]
#![feature(total_cmp)]

pub mod api;
pub mod response;
pub mod node_client;
mod opt_core;

#[macro_use]
extern crate rocket;
extern crate rocket_contrib;
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


use rocket_contrib::json::{Json, JsonValue};
use rocket::response::Responder;
use api::{ToDo, OptRequest};
use response::OptResponse;
use serde::__private::ptr::null;


#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/todos")]
pub fn todos() -> Json<Vec<ToDo>> {
    Json(vec![ToDo {
        id: 1,
        title: "Read Rocket tutorial".into(),
        description: "Read https://rocket.rs/guide/quickstart/".into(),
        done: false,
    }])
}

#[post("/todos", data = "<todo>")]
pub fn new_todo(todo: Json<ToDo>) -> Json<ToDo> {
    Json(todo.0)
}

#[post("/opt_swap", data = "<req>")]
pub fn opt_swap(req: Json<OptRequest>) -> Json<OptResponse> {
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
        .mount("/", routes![index, todos, new_todo,opt_swap])
        .launch();
    print!("end")
}
