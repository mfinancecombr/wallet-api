use chrono::{DateTime, Duration, Local, TimeZone, Utc};
use mongodb::coll::options::FindOptions;
use mongodb::db::ThreadedDatabase;
use mongodb::{Bson, bson, doc};
use rocket_okapi::{openapi};
use serde::{Deserialize, Serialize};
use yahoo_finance::{history, Bar};

use crate::error::{BackendError};
use crate::walletdb::WalletDB;


#[derive(Clone, Debug, Serialize, Deserialize)]
struct AssetDay {
    symbol: String,
    time: DateTime<Utc>,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: i64,
}

impl From<Bar> for AssetDay {
    fn from(bar: Bar) -> AssetDay {
        AssetDay {
            symbol: String::new(),
            time: bar.timestamp,
            open: bar.open,
            high: bar.high,
            low: bar.low,
            close: bar.close,
            volume: bar.volume as i64
        }
    }
}

#[openapi]
#[post("/assets/refresh/<symbol>")]
pub fn refresh_historical_for_symbol(db: WalletDB, symbol: String) -> Result<(), BackendError> {
    do_refresh_for_symbol(&*db, &symbol)
}

fn do_refresh_for_symbol(wallet: &mongodb::db::Database, symbol: &str) -> Result<(), BackendError> {
    let mut since = DateTime::<Utc>::from(Local.ymd(2006, 1, 1).and_hms(0, 0, 0));

    // First check if we need to override our since constraint, as we may
    // already have downloaded some historical data, and we don't want to
    // lose any of the earlier ones when the API moves its availability window.
    let mut options = FindOptions::new();
    options.sort = Some(doc!{ "time": -1 });
    wallet.collection("historical").find_one(Some(doc!{ "symbol": symbol }), Some(options))
        .map(|document| {
            if let Some(document) = document {
                let asset_day: Result<AssetDay, _> = bson::from_bson(Bson::Document(document));
                if let Ok(asset_day) = asset_day {
                    // The range for yahoo_finance is inclusive and a bit weird, as it seems
                    // to disregard the time(?). To avoid duplicating the last day we have,
                    // we tell it to start from the next day.
                    since = asset_day.time.date().and_hms(0, 0, 0) + Duration::days(1);
                }
            }
        }).ok();


    // Limit the range to yesterday, so we don't keep adding several times for
    // today in case we get called multiple times.
    let yesterday = DateTime::<Utc>::from(Local::today().and_hms(23, 59, 59) - Duration::days(1));
    if yesterday < since || yesterday.date() == since.date() {
        return Ok(())
    }

    let data = history::retrieve_range(
        &format!("{}.SA", symbol),
        since,
        Some(yesterday)
    ).map_err(|e| BackendError::Yahoo(format!("{:?}", e)))?;

    let mut docs = Vec::<bson::ordered::OrderedDocument>::new();
    for bar in data {
        let mut asset_day = AssetDay::from(bar);
        asset_day.symbol = symbol.to_string();

        let doc = bson::to_bson(&asset_day)
            .map_err(|e| BackendError::Bson(format!("{:?}", e)))?;

        let doc = doc.as_document()
            .ok_or(BackendError::Bson(format!("Could not convert to Document")))?;

        docs.push(doc.clone());
    }

    wallet.collection("historical").insert_many(docs, None)
        .map_err(|e| BackendError::Database(format!("{:?}", e)))
        .map(|_| ())
}

#[cfg(test)]
mod tests {
    use chrono::Datelike;
    use mongodb::ThreadedClient;

    use super::*;


    #[test]
    fn repeated_refreshes() {
        let db_client = mongodb::Client::with_uri("mongodb://127.0.0.1:27017/")
            .expect("Could not connect to mongodb");
        let db = db_client.db("wallet-fake-test");
        let collection = db.collection("historical");

        assert_eq!(collection.delete_many(doc!{}, None).is_ok(), true);

        // Downloading the data...
        let result = do_refresh_for_symbol(&db, "ANIM3");
        assert_eq!(result.is_ok(), true);

        // Did we add some stuff?
        let original_count = collection.count(None, None).expect("Count failed");
        assert!(original_count > 0);

        // Delete the last year.
        let filter = doc!{
            "time": { "$gt": format!("{}-1-1", Local::today().year() - 1) }
        };
        collection.delete_many(filter, None).expect("Delete many failed");

        // Make sure we actually deleted something, but still have a bit.
        let count = collection.count(None, None).expect("Count failed");
        assert!(count > 0 && count < original_count);

        // Refresh again.
        let result = do_refresh_for_symbol(&db, "ANIM3");
        assert_eq!(result.is_ok(), true);

        // Do we get to the same number we had at the first run?
        let count = collection.count(None, None).expect("Count failed");
        assert_eq!(count, original_count);

        // Refresh yet again.
        let result = do_refresh_for_symbol(&db, "ANIM3");
        assert_eq!(result.is_ok(), true);

        // Do we still get to the same number we had at the first run?
        let count = collection.count(None, None).expect("Count failed");
        assert_eq!(count, original_count);
    }
}