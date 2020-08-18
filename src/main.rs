#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_okapi;
extern crate rocket_cors;
use rocket_okapi::swagger_ui::*;

mod broker;
mod error;
mod operation;
mod position;
mod rest;
mod stock;
mod walletdb;

use broker::*;
use stock::*;
use walletdb::WalletDB;


fn main() {
    let mut cors = rocket_cors::CorsOptions::default();
    cors.expose_headers.insert(String::from("X-Total-Count"));

    let cors = cors.to_cors()
        .expect("Failed to create CORS configuration");

    rocket::ignite()
        .mount(
            "/",
            routes_with_openapi![
                add_broker,
                get_brokers,
                get_broker_by_oid,
                update_broker_by_oid,
                delete_broker_by_oid,

                add_stock_operation,
                get_stock_operations,
                get_stock_operation_by_oid,
                update_stock_operation_by_oid,
                delete_stock_operation_by_oid,
                get_stock_position_by_symbol,
            ],
        )
        .mount(
            "/swagger-ui/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .attach(WalletDB::fairing())
        .attach(cors)
        .launch();
}