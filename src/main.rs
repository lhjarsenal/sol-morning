#![feature(proc_macro_hygiene)]
#![feature(decl_macro)]

mod api;
mod response;

#[macro_use]
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
extern crate anyhow;
extern crate solana_program;
extern crate rust_decimal;

use rocket_contrib::json::{Json, JsonValue};
use rocket::response::Responder;
use api::{ToDo, OptRequest};
use response::OptResponse;


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

#[post("/opt_swap", data = "<todo>")]
pub fn opt_swap(todo: Json<OptRequest>) -> Json<OptResponse> {
    println!("todo={:?}", todo.0);

    todo.0.loadData();
    Json(OptResponse::new())
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index, todos, new_todo,opt_swap])
        .launch();
    print!("end")
}
