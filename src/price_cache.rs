use futures::{future, StreamExt};
use log::debug;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::Rocket;
use std::collections::HashMap;
use std::sync::Mutex;
use yahoo_finance::Streamer;

use crate::event::get_distinct_symbols;

struct PriceMap(HashMap<String, f64>);
impl PriceMap {
    pub fn new() -> Self {
        PriceMap(HashMap::<String, f64>::new())
    }
}

lazy_static! {
    static ref PRICE_CACHE: Mutex<PriceMap> = Mutex::new(PriceMap::new());
}

pub struct PriceCache {}

impl PriceCache {
    pub fn fairing() -> Self {
        PriceCache {}
    }

    #[cfg(not(test))]
    pub fn get_current_price(symbol: &str) -> Option<f64> {
        PRICE_CACHE
            .lock()
            .map(|price_cache| price_cache.0.get(symbol).copied())
            .ok()
            .flatten()
    }

    pub fn update_current_price(symbol: String, price: f64) {
        PRICE_CACHE
            .lock()
            .map(|mut price_cache| {
                price_cache.0.insert(symbol, price);
            })
            .expect("Failed to lock price cache map");
    }

    #[tokio::main]
    async fn watch_prices(symbols: Vec<&str>) {
        println!("PREPARING TO STREAM {:?}", symbols);
        let streamer = Streamer::new(symbols);
        let _ = std::panic::catch_unwind(async move || loop {
            streamer
                .stream()
                .await
                .for_each(|quote| {
                    debug!(
                        "At {}, {} is trading for ${}",
                        quote.timestamp, quote.symbol, quote.price
                    );

                    let mut symbol = quote.symbol.to_string();

                    // Remove the .SA.
                    symbol.truncate(symbol.len() - 3);

                    PriceCache::update_current_price(symbol, quote.price);

                    future::ready(())
                })
                .await;
        });
    }
}

impl Fairing for PriceCache {
    fn info(&self) -> Info {
        Info {
            name: "PriceCache",
            kind: Kind::Launch,
        }
    }

    fn on_launch(&self, _rocket: &Rocket) {
        let mut symbols = get_distinct_symbols(None).expect("Failed to query mongodb for symbols");
        println!("LAUNCHING LIVE");
        std::thread::spawn(move || {
            println!("LAUNCHING LIVE2");
            Self::watch_prices(
                symbols
                    .iter_mut()
                    .map(|s| {
                        s.push_str(".SA");
                        String::as_str(s)
                    })
                    .collect::<Vec<&str>>(),
            );
        });
    }
}
