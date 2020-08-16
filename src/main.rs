#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_okapi;
use rocket_okapi::swagger_ui::*;

mod broker;
mod error;
mod operation;
mod position;
mod stock;
mod walletdb;

use broker::*;
use stock::*;
use walletdb::WalletDB;


fn main() {
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
        .launch();
}