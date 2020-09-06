#![warn(clippy::all)]
#![feature(proc_macro_hygiene, decl_macro, async_closure)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_okapi;
extern crate rocket_cors;
use rocket_okapi::swagger_ui::*;

mod broker;
#[macro_use]
mod error;
mod historical;
mod operation;
mod portfolio;
mod position;
mod rest;
mod scheduling;
mod stock;
mod walletdb;

use broker::*;
use historical::*;
use portfolio::*;
use scheduling::Scheduler;
use stock::*;
use walletdb::WalletDB;

fn main() {
    let mut cors = rocket_cors::CorsOptions::default();
    cors.expose_headers.insert(String::from("X-Total-Count"));

    let cors = cors.to_cors().expect("Failed to create CORS configuration");

    rocket::ignite()
        .mount(
            "/api/v1/",
            routes_with_openapi![
                // Broker
                add_broker,
                get_brokers,
                get_broker_by_oid,
                update_broker_by_oid,
                delete_broker_by_oid,
                // Stocks
                add_stock_operation,
                get_stock_operations,
                get_stock_operation_by_oid,
                update_stock_operation_by_oid,
                delete_stock_operation_by_oid,
                get_stock_position_by_symbol,
                // Historical
                refresh_historicals,
                refresh_historical_for_symbol,
                // Portfolio
                add_portfolio,
                get_portfolios,
                get_portfolio_by_oid,
                update_portfolio_by_oid,
                delete_portfolio_by_oid,
                portfolio_position,
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
        .attach(Scheduler::fairing())
        .attach(cors)
        .launch();
}
