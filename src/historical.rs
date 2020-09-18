use chrono::{DateTime, Duration, NaiveDateTime, TimeZone, Utc};
use mongodb::bson::{doc, from_bson, to_bson, Bson, Document};
use mongodb::options::FindOneOptions;
use rayon::prelude::*;
use rocket_okapi::openapi;
use serde::{Deserialize, Serialize};
use yahoo_finance::{history, Bar};

use crate::error::{BackendError, WalletResult};
use crate::event::get_distinct_symbols;
use crate::scheduling::LockMap;
use crate::walletdb::{Queryable, WalletDB};

#[cfg(not(test))]
use chrono::Date;

#[cfg(test)]
pub mod test;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssetDay {
    pub symbol: String,
    pub time: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
}

impl Queryable for AssetDay {
    fn collection_name() -> &'static str {
        "historical"
    }
}

impl From<Bar> for AssetDay {
    fn from(bar: Bar) -> AssetDay {
        AssetDay {
            symbol: String::new(),
            time: DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp((bar.timestamp / 1000) as i64, 0),
                Utc,
            ),
            open: bar.open,
            high: bar.high,
            low: bar.low,
            close: bar.close,
            volume: bar.volume.unwrap_or(0) as i64,
        }
    }
}

/// # Triggers a full refresh of historical data
///
/// Triggers a full refresh of historical price data for all assets present in the
/// database. Does not return data.
#[openapi]
#[post("/historicals/refresh")]
pub fn refresh_historicals() -> WalletResult<()> {
    Historical::refresh_all()
}

/// # Triggers a full refresh of historical data for a symbol
///
/// Triggers a full refresh of historical price data for a symbol. Does not return data.
#[openapi]
#[post("/historicals/refresh/<symbol>")]
pub fn refresh_historical_for_symbol(symbol: String) -> WalletResult<()> {
    do_refresh_for_symbol(&symbol)
}

pub struct Historical {}

impl Historical {
    pub fn refresh_all() -> WalletResult<()> {
        let symbols = get_distinct_symbols(None)?;

        symbols
            .into_par_iter()
            .try_for_each::<_, WalletResult<_>>(|symbol| {
                do_refresh_for_symbol(&symbol)?;
                Ok(())
            })?;

        Ok(())
    }

    #[cfg(not(test))]
    pub fn current_price_for_symbol(symbol: String) -> f64 {
        let asset_day = Historical::get_for_day_with_fallback(&symbol, Utc::today());
        if let Ok(asset_day) = asset_day {
            asset_day.close
        } else {
            f64::NAN
        }
    }

    #[cfg(not(test))]
    pub fn get_for_day_with_fallback(symbol: &str, date: Date<Utc>) -> WalletResult<AssetDay> {
        let db = WalletDB::get_connection();
        let historical = db.collection("historical");

        // We search for historical prices over a week to make sure we get
        // data even through weekends and holidays.
        // FIXME: this version of the mongodb driver doesn't seem to like
        // DateTime<Utc> objects. Newer ones work, maybe bite the bullet here.
        let range_from = (date - Duration::days(7)).and_hms(0, 0, 0).to_rfc3339();
        let range_to = date.and_hms(23, 59, 59).to_rfc3339();

        let filter = doc! {
            "$and": [
                { "symbol": symbol.to_string() },
                { "time": { "$gte": range_from } },
                { "time": { "$lte": range_to } },
            ]
        };

        let find_options = FindOneOptions::builder().sort(doc! { "time": -1 });

        let document = historical.find_one(filter, find_options.build())?;

        if let Some(document) = document {
            Ok(from_bson::<AssetDay>(Bson::Document(document))?)
        } else {
            Err(BackendError::NotFound)
        }
    }
}

#[tokio::main]
async fn do_refresh_for_symbol(symbol: &str) -> WalletResult<()> {
    // Ensure we do not try to refresh the same symbol more than once at a time.
    let _guard = LockMap::lock("historical", symbol);

    let mut since = Utc.ymd(2006, 1, 1).and_hms(0, 0, 0);

    // First check if we need to override our since constraint, as we may
    // already have downloaded some historical data, and we don't want to
    // lose any of the earlier ones when the API moves its availability window.
    let db = WalletDB::get_connection();
    let options = FindOneOptions::builder().sort(doc! { "time": -1 });
    db.collection("historical")
        .find_one(doc! { "symbol": symbol }, options.build())
        .map(|document| {
            if let Some(document) = document {
                let asset_day: Result<AssetDay, _> = from_bson(Bson::Document(document));
                if let Ok(asset_day) = asset_day {
                    // The range for yahoo_finance is inclusive and a bit weird, as it seems
                    // to disregard the time(?). To avoid duplicating the last day we have,
                    // we tell it to start from the next day.
                    since = asset_day.time.date().and_hms(0, 0, 0) + Duration::days(1);
                }
            }
        })
        .ok();

    // Limit the range to yesterday, so we don't keep adding several times for
    // today in case we get called multiple times.
    let yesterday = Utc::today().and_hms(23, 59, 59) - Duration::days(1);
    if yesterday < since || yesterday.date() == since.date() {
        return Ok(());
    }

    let data = history::retrieve_range(&format!("{}.SA", symbol), since, Some(yesterday)).await;

    // HACK: yahoo-finance-rs will fail on queries for days with no data
    // and it doesn't provide a good way of understanding what kind of error
    // happened.
    let data = match data {
        Ok(data) => data,
        Err(e) => {
            if format!("{:?}", e).contains("BadData {") {
                Vec::<Bar>::new()
            } else {
                return Err(dang!(Yahoo, format!("{}: {}", symbol, e)));
            }
        }
    };

    let mut docs = Vec::<Document>::new();
    for bar in data {
        let mut asset_day = AssetDay::from(bar);
        asset_day.symbol = symbol.to_string();

        // HACK: yahoo-finance-rs will sometimes return one bar from the day
        // before the one specified as the start of the range. We do this
        // sanity check here to avoid that.
        // See https://github.com/fbriden/yahoo-finance-rs/issues/25
        if asset_day.time < since {
            continue;
        }

        let doc = match to_bson(&asset_day)? {
            Bson::Document(doc) => Ok(doc),
            _ => Err(dang!(Bson, "Could not convert to Document")),
        }?;

        docs.push(doc);
    }

    if docs.is_empty() {
        return Ok(());
    }

    db.collection("historical").insert_many(docs, None)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::Datelike;

    use super::*;

    #[test]
    fn repeated_refreshes() {
        WalletDB::init_client("mongodb://localhost:27017/");

        let db = WalletDB::get_connection();

        let collection = db.collection("historical");

        assert_eq!(collection.delete_many(doc! {}, None).is_ok(), true);

        // Downloading the data...
        let result = do_refresh_for_symbol("ANIM3");
        assert_eq!(result.is_ok(), true);

        // Did we add some stuff?
        let original_count = collection
            .count_documents(None, None)
            .expect("Count failed");
        assert!(original_count > 0);

        // Delete the last year.
        let filter = doc! {
            "time": { "$gt": format!("{}-1-1", Utc::today().year() - 1) }
        };
        collection
            .delete_many(filter, None)
            .expect("Delete many failed");

        // Make sure we actually deleted something, but still have a bit.
        let count = collection
            .count_documents(None, None)
            .expect("Count failed");
        assert!(count > 0 && count < original_count);

        // Refresh again.
        let result = do_refresh_for_symbol("ANIM3");
        assert_eq!(result.is_ok(), true);

        // Do we get to the same number we had at the first run?
        let count = collection
            .count_documents(None, None)
            .expect("Count failed");
        assert_eq!(count, original_count);

        // Refresh yet again.
        let result = do_refresh_for_symbol("ANIM3");
        assert_eq!(result.is_ok(), true);

        // Do we still get to the same number we had at the first run?
        let count = collection
            .count_documents(None, None)
            .expect("Count failed");
        assert_eq!(count, original_count);

        if let Err(e) = db.drop(None) {
            println!("Failed to drop test db {}", format!("{:?}", e));
        }
    }
}
