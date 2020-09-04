use chrono::{Date, DateTime, Datelike, Duration, Local, TimeZone, Utc, Weekday};
use log::{debug, info, warn};
use mongodb::bson::{doc, from_bson, Bson, Document};
use mongodb::options::{FindOneOptions, FindOptions};
use rayon::prelude::*;
use rocket_okapi::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::sync::Mutex;

use crate::error::*;
use crate::historical::Historical;
use crate::operation::{get_distinct_symbols, BaseOperation, OperationKind};
use crate::scheduling::LockMap;
use crate::walletdb::*;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct Position {
    pub symbol: String,
    pub average_price: f64,
    pub cost_basis: f64,
    pub quantity: i64,
    pub time: DateTime<Local>,
    pub current_price: f64,
    pub gain: f64,
    pub realized: f64,
    pub sales: Vec<Sale>,
}

impl Position {
    fn new(symbol: &str) -> Self {
        Position {
            symbol: symbol.to_string(),
            cost_basis: 0.0,
            quantity: 0,
            average_price: 0.0,
            time: Local::now(),
            current_price: 0.0,
            gain: 0.0,
            realized: 0.0,
            sales: Vec::<Sale>::new(),
        }
    }
}

impl Queryable for Position {
    fn collection_name() -> &'static str {
        "positions"
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct Sale {
    pub time: DateTime<Local>,
    pub quantity: i64,
    pub cost_price: f64,
    pub sell_price: f64,
}

fn get_safely<T>(doc: &Document, key: &str) -> WalletResult<T>
where
    T: DeserializeOwned,
{
    if let Some(value) = doc.get(key) {
        from_bson::<T>(value.clone()).map_err(|e| dang!(Bson, e))
    } else {
        Err(dang!(
            Database,
            format!("field `{}` not found on document", key)
        ))
    }
}

fn find_all_fridays_between(from: DateTime<Local>, to: DateTime<Local>) -> Vec<Date<Utc>> {
    let mut fridays = Vec::<Date<Utc>>::new();
    let mut date = from.date();
    let to = to.date();

    while date < to {
        if date.weekday() == Weekday::Fri {
            fridays.push(date.with_timezone(&Utc));
            date = date + Duration::days(7);
        } else {
            date = date + Duration::days(1);
        }
    }

    fridays
}

#[tokio::main]
async fn do_calculate_for_symbol(symbol: String) -> WalletResult<Position> {
    // Ensure we do not try to calculate for the same symbol more than once at a time.
    // Create it here so it is locked even before the thread gets to run, to avoid
    // races with callers of this function or multiple calls of this function.
    let guard = LockMap::lock(Position::collection_name(), &symbol);

    let db = WalletDB::get_connection();
    let collection = db.collection(BaseOperation::collection_name());

    let mut date_from = Utc.timestamp(61, 0);

    // If we already have a bunch of position snapshots, we pick up
    // from the last one rather than starting from scratch.
    let mut position = Position::last(&symbol)
        .map(|pos| {
            date_from = pos.time.with_timezone(&Utc);
            pos
        })
        .unwrap_or_else(|| Position::new(&symbol));

    let filter = doc! {
        "$and": [
            { "symbol": &symbol },
            {
                "time": {
                    "$lte": Local::today().and_hms(23, 59, 59)
                        .with_timezone(&Utc).to_rfc3339()
                }
            },
            {
                "time": {
                    "$gt": date_from.to_rfc3339()
                }
            }
        ]
    };

    let options = FindOptions::builder().sort(doc! { "time": 1 });
    let cursor = match collection.find(filter, options.build()) {
        Ok(cursor) => cursor,
        Err(e) => {
            return Err(dang!(Database, e));
        }
    };

    let mut references = Vec::<Position>::new();
    for document in cursor {
        if let Ok(document) = document {
            position.time = get_safely::<DateTime<Utc>>(&document, "time")?.with_timezone(&Local);

            let quantity = get_safely::<i64>(&document, "quantity")?;
            let kind = get_safely::<OperationKind>(&document, "type")?;
            match kind {
                OperationKind::Purchase => {
                    let price = get_safely::<f64>(&document, "price")?;
                    position.cost_basis += price * quantity as f64;
                    position.quantity += quantity;
                }
                OperationKind::Sale => {
                    /* When selling we need to use the average price at the moment
                     * of the sale for the average calculation to work. We may
                     * take out too little if the current price is lower or too
                     * much, otherwise.
                     */
                    let cost_price = position.cost_basis / position.quantity as f64;
                    position.cost_basis -= cost_price * quantity as f64;
                    position.quantity -= quantity;

                    let sell_price = get_safely::<f64>(&document, "price")?;
                    position.realized +=
                        quantity as f64 * sell_price - quantity as f64 * cost_price;

                    position.sales.push(Sale {
                        time: position.time.with_timezone(&Local),
                        quantity,
                        cost_price,
                        sell_price,
                    })
                }
            }

            if position.quantity != 0 && position.cost_basis != 0.0 {
                position.average_price = position.cost_basis / position.quantity as f64;
            }

            references.push(position.clone());
        }
    }

    // Up to here we used the time for the last operation, but we have been asked
    // for the "current" position. We also need to add that to references' last position,
    // so that the snapshots will be calculated up to today.
    position.time = Local::now();
    references.push(position.clone());

    std::thread::spawn(move || {
        let _guard = guard;
        Position::create_snapshots(&symbol, references).map_err(|e| {
            warn!("failure saving references: {:?}", e);
            e
        })
    });

    Ok(position)
}

impl Position {
    pub fn last(symbol: &str) -> Option<Self> {
        let db = WalletDB::get_connection();
        let collection = db.collection(Position::collection_name());

        let filter = doc! { "symbol": symbol.to_string() };
        let options = FindOneOptions::builder().sort(doc! { "time": -1 }).build();

        if let Ok(doc) = collection.find_one(filter, options) {
            doc.map(|doc| from_bson(Bson::Document(doc)).ok())
                .unwrap_or(None)
        } else {
            None
        }
    }

    pub fn calculate_for_symbol(symbol: &str) -> WalletResult<Position> {
        // Ensure we do not try to calculate for the same symbol more than once at a time.
        let _guard = LockMap::lock(BaseOperation::collection_name(), symbol);

        // Fire a background thread to get the current price.
        let ysymbol = symbol.to_string();
        let current_price =
            std::thread::spawn(move || Historical::current_price_for_symbol(ysymbol));

        let dsymbol = symbol.to_string();
        let mut position = std::thread::spawn(move || do_calculate_for_symbol(dsymbol))
            .join()
            .unwrap()?;

        let current_price = current_price.join().unwrap();
        position.current_price = current_price;
        position.gain = current_price * position.quantity as f64 - position.cost_basis;

        Ok(position)
    }

    pub fn calculate_all() -> WalletResult<Vec<Position>> {
        let positions = Mutex::new(Vec::<Position>::new());

        let symbols = get_distinct_symbols()?;
        symbols
            .into_par_iter()
            .try_for_each::<_, WalletResult<_>>(|symbol| {
                let position = Position::calculate_for_symbol(&symbol)?;
                positions.lock().unwrap().push(position);
                Ok(())
            })?;

        Ok(positions.into_inner().unwrap())
    }

    pub fn create_snapshots(symbol: &str, mut references: Vec<Position>) -> WalletResult<()> {
        info!("[{}] saving Position snapshots", symbol);

        let mut previous_position: Option<Position> = None;
        for position in &mut references {
            if let Some(mut previous_position) = previous_position {
                info!(
                    "[{}] generating snapshots for range {:?} -> {:?}",
                    symbol, previous_position.time, position.time
                );
                for friday in find_all_fridays_between(previous_position.time, position.time) {
                    let asset_day = Historical::get_for_day_with_fallback(symbol, friday);
                    if let Ok(asset_day) = asset_day {
                        previous_position.time = asset_day.time.with_timezone(&Local);
                        previous_position.current_price = asset_day.close;
                    } else {
                        warn!(
                            "failed to find historical data for {} on {}",
                            symbol, friday
                        );
                        previous_position.time = friday.with_timezone(&Local).and_hms(12, 0, 0);
                    }

                    previous_position.gain = previous_position.current_price
                        * previous_position.quantity as f64
                        - previous_position.cost_basis;

                    debug!("[{}] inserting snapshot {:?}", symbol, previous_position);
                    insert_one(previous_position.clone())?;
                }
            }

            previous_position = Some(position.clone());
        }
        info!("[{}] done saving Position snapshots", symbol);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use std::vec::Vec;

    use super::*;
    use crate::operation::{AssetKind, OperationKind};
    use crate::stock::StockOperation;

    #[test]
    fn position_calculation() {
        WalletDB::init_client("mongodb://localhost:27017/");

        let db = WalletDB::get_connection();

        assert!(
            db.collection(BaseOperation::collection_name())
                .delete_many(doc! {}, None)
                .is_ok(),
            true
        );

        assert!(
            db.collection(Position::collection_name())
                .delete_many(doc! {}, None)
                .is_ok(),
            true
        );

        let symbol = String::from("FAKE4");

        let mut stock = StockOperation {
            asset_kind: AssetKind::Stock,
            operation: BaseOperation {
                id: None,
                kind: OperationKind::Purchase,
                broker: String::from("FakeTestBroker"),
                portfolios: vec![String::from("FakeTestWallet")],
                symbol: symbol.clone(),
                time: Local.ymd(2020, 1, 1).and_hms(12, 0, 0),
                price: 10.0,
                quantity: 100,
                fees: 0.0,
            },
        };

        let mut sales = Vec::<Sale>::new();

        assert!(insert_one(stock.clone()).is_ok(), true);

        stock.operation.price = 12.0;
        stock.operation.quantity = 50;
        stock.operation.kind = OperationKind::Sale;
        stock.operation.time = Local.ymd(2020, 2, 1).and_hms(12, 0, 0);

        sales.push(Sale {
            time: stock.operation.time,
            quantity: stock.operation.quantity,
            cost_price: 10.0,
            sell_price: stock.operation.price,
        });

        assert!(insert_one(stock.clone()).is_ok(), true);

        stock.operation.price = 4.0;
        stock.operation.kind = OperationKind::Purchase;
        stock.operation.time = Local.ymd(2020, 3, 1).and_hms(12, 0, 0);

        assert!(insert_one(stock.clone()).is_ok(), true);

        // This is a Friday, so will test corner cases of the position snapshots.
        stock.operation.price = 10.0;
        stock.operation.kind = OperationKind::Purchase;
        stock.operation.time = Local.ymd(2020, 3, 27).and_hms(12, 0, 0);

        assert!(insert_one(stock).is_ok(), true);

        // Do a full update first, which should trigger calculation for our
        // FAKE4. This means the specific call below should start from an
        // existing reference.
        Position::calculate_all().expect("Something went wrong");

        let position = Position::calculate_for_symbol("FAKE4");
        assert_eq!(position.is_ok(), true);
        let position = position.unwrap();

        let same_position = Position::calculate_for_symbol("FAKE4");
        assert_eq!(same_position.is_ok(), true);
        let same_position = same_position.unwrap();

        assert_relative_eq!(position.cost_basis, same_position.cost_basis,);
        assert_eq!(position.quantity, same_position.quantity);
        assert_relative_eq!(position.average_price, same_position.average_price);
        assert_relative_eq!(position.realized, same_position.realized);
        assert_eq!(position.sales, same_position.sales);

        // Manually check that the time is pretty close to now, since we will update our
        // reference below with what we got.
        assert!(Local::now() - position.time < Duration::seconds(10));

        // NOTE: Our Historical mock for now just returns a static 9.0 price for all requests.
        assert_eq!(
            position,
            Position {
                symbol,
                average_price: 8.0,
                cost_basis: 1200.0,
                quantity: 150,
                time: position.time,
                current_price: 9.0,
                gain: 150.0,
                realized: 100.0,
                sales,
            }
        );

        let collection = db.collection(Position::collection_name());

        // Snapshots should go all the way to "today", so we select a small
        // known sample to verify everything looks ok.
        let filter = doc! {
            "time": { "$lt": "2020-04-04" }
        };

        let positions = collection
            .find(Some(filter), None)
            .map(|cursor| Position::from_docs(cursor).expect("Failed to convert document"))
            .expect("Failed to query positions collection");

        assert_eq!(positions.len(), 14);

        // time, cost_basis, quantity, realized, gain
        let expected = vec![
            ("2020-01-03", 1000.0, 100, 0.0, -100.0),
            ("2020-01-10", 1000.0, 100, 0.0, -100.0),
            ("2020-01-17", 1000.0, 100, 0.0, -100.0),
            ("2020-01-24", 1000.0, 100, 0.0, -100.0),
            ("2020-01-31", 1000.0, 100, 0.0, -100.0),
            ("2020-02-07", 500.0, 50, 100.0, -50.0),
            ("2020-02-14", 500.0, 50, 100.0, -50.0),
            ("2020-02-21", 500.0, 50, 100.0, -50.0),
            ("2020-02-28", 500.0, 50, 100.0, -50.0),
            ("2020-03-06", 700.0, 100, 100.0, 200.0),
            ("2020-03-13", 700.0, 100, 100.0, 200.0),
            ("2020-03-20", 700.0, 100, 100.0, 200.0),
            ("2020-03-27", 1200.0, 150, 100.0, 150.0),
            ("2020-04-03", 1200.0, 150, 100.0, 150.0),
        ];

        for (index, position) in positions.into_iter().enumerate() {
            let (time, cost_basis, quantity, realized, gain) = &expected[index];
            assert_eq!(*time, position.time.naive_local().date().to_string());
            assert_relative_eq!(*cost_basis, position.cost_basis);
            assert_eq!(*quantity, position.quantity);
            assert_relative_eq!(*realized, position.realized);
            assert_relative_eq!(*gain, position.gain);
        }
    }
}
