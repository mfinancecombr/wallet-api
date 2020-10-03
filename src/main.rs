#![warn(clippy::all)]
#![feature(proc_macro_hygiene, decl_macro, async_closure, try_trait)]

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
mod event;
mod historical;
mod operation;
mod portfolio;
mod position;
mod price_cache;
mod rest;
mod scheduling;
mod stock;
mod walletdb;
mod x_response_time;

use broker::*;
use event::*;
use historical::*;
use portfolio::*;
use price_cache::PriceCache;
use scheduling::Scheduler;
use stock::*;
use walletdb::WalletDB;
use x_response_time::RequestTimer;

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
                // Events
                add_event,
                get_events,
                get_event_by_oid,
                update_event_by_oid,
                delete_event_by_oid,
                // Stock
                get_stock_position_by_symbol,
                // Historical
                refresh_historicals,
                refresh_historical_for_symbol,
                // Performance
                performance,
                // Position
                positions,
                // Portfolio
                add_portfolio,
                get_portfolios,
                get_portfolio_by_oid,
                update_portfolio_by_oid,
                delete_portfolio_by_oid,
                portfolio_positions,
            ],
        )
        .mount(
            "/swagger-ui/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../api/v1/openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .attach(RequestTimer)
        .attach(WalletDB::fairing())
        .attach(PriceCache::fairing())
        .attach(Scheduler::fairing())
        .attach(cors)
        .launch();
}
