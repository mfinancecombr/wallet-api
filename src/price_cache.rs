use rocket::fairing::{Fairing, Info, Kind};
use rocket::Rocket;
use std::collections::HashMap;
use std::sync::Mutex;

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

    #[cfg(not(test))]
    pub fn update_current_price(symbol: String, price: f64) {
        PRICE_CACHE
            .lock()
            .map(|mut price_cache| {
                price_cache.0.insert(symbol, price);
            })
            .expect("Failed to lock price cache map");
    }
}

impl Fairing for PriceCache {
    fn info(&self) -> Info {
        Info {
            name: "PriceCache",
            kind: Kind::Launch,
        }
    }

    fn on_launch(&self, _rocket: &Rocket) {}
}
