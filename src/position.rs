use chrono::{Date, DateTime, Datelike, Duration, Local, TimeZone, Utc, Weekday};
use log::{debug, info, warn};
use mongodb::coll::options::FindOptions;
use mongodb::db::ThreadedDatabase;
use mongodb::{bson, doc, Bson};
use rayon::prelude::*;
use rocket_okapi::JsonSchema;
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::sync::Mutex;
use yahoo_finance::history;

use crate::error::*;
use crate::historical::AssetDay;
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

impl<'de> Queryable<'de> for Position {
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

fn get_safely<'de, T>(doc: &bson::ordered::OrderedDocument, key: &str) -> WalletResult<T>
where
    T: Deserialize<'de>,
{
    if let Some(value) = doc.get(key) {
        bson::from_bson::<T>(value.clone()).map_err(|e| dang!(Bson, e))
    } else {
        Err(dang!(
            Database,
            format!("field `{}` not found on document", key)
        ))
    }
}

#[tokio::main]
async fn current_price_for_symbol(symbol: String, date_to: DateTime<Local>) -> f64 {
    let ysymbol = format!("{}.SA", &symbol);
    let date_from = date_to.date().and_hms(0, 0, 0);
    let bar = history::retrieve_range(
        &ysymbol,
        DateTime::<Utc>::from(date_from),
        Some(DateTime::<Utc>::from(date_to)),
    )
    .await
    .ok()
    .and_then(|mut bar| bar.pop());

    if let Some(bar) = bar {
        bar.close
    } else {
        f64::NAN
    }
}

fn find_all_fridays_between(from: DateTime<Local>, to: DateTime<Local>) -> Vec<Date<Utc>> {
    let mut fridays = Vec::<Date<Utc>>::new();
    let mut date = DateTime::<Utc>::from(from).date();
    let to = DateTime::<Utc>::from(to).date();

    while date < to {
        if date.weekday() == Weekday::Fri {
            fridays.push(date);
            date = date + Duration::days(7);
        } else {
            date = date + Duration::days(1);
        }
    }

    fridays
}

#[tokio::main]
async fn do_calculate_for_symbol(
    db: &mongodb::db::Database,
    symbol: String,
) -> WalletResult<Position> {
    let collection = db.collection(BaseOperation::collection_name());

    let mut date_from = Utc.timestamp(61, 0);

    // If we already have a bunch of position snapshots, we pick up
    // from the last one rather than starting from scratch.
    let mut position = Position::last(db, &symbol)
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

    let options = Some(FindOptions::new()).map(|mut options| {
        options.sort = Some(doc! { "time": 1 });
        options
    });

    let cursor = match collection.find(Some(filter), options) {
        Ok(cursor) => cursor,
        Err(e) => {
            return Err(dang!(Database, e));
        }
    };

    let mut snapshots = Vec::<Position>::new();
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
                    position.realized += quantity as f64 * sell_price;

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

            // Note that save_snapshots disregards the first item on this vector.
            snapshots.push(position.clone());
        }
    }

    std::thread::spawn(move || {
        let db = WalletDB::get_connection();
        Position::save_snapshots(&db, &symbol, snapshots).map_err(|e| {
            warn!("failure saving snapshots: {:?}", e);
            e
        })
    });

    Ok(position)
}

impl Position {
    pub fn last(db: &mongodb::db::Database, symbol: &str) -> Option<Self> {
        let collection = db.collection(Position::collection_name());

        let filter = doc! { "symbol": symbol.to_string() };

        let options = Some(FindOptions::new()).map(|mut options| {
            options.sort = Some(doc! { "time": -1 });
            options
        });

        if let Ok(doc) = collection.find_one(Some(filter), options) {
            doc.map(|doc| bson::from_bson(Bson::Document(doc)).ok())
                .unwrap_or(None)
        } else {
            None
        }
    }

    pub fn calculate_for_symbol(
        db: &mongodb::db::Database,
        symbol: &str,
    ) -> WalletResult<Position> {
        // Ensure we do not try to calculate for the same symbol more than once at a time.
        let _guard = LockMap::lock(BaseOperation::collection_name(), symbol);

        // Fire a background thread to get the current price.
        let ysymbol = symbol.to_string();
        let current_price = std::thread::spawn(move || {
            current_price_for_symbol(ysymbol, Local::today().and_hms(23, 59, 59))
        });

        let db = db.clone();
        let dsymbol = symbol.to_string();
        let mut position = std::thread::spawn(move || do_calculate_for_symbol(&db, dsymbol))
            .join()
            .unwrap()?;

        let current_price = current_price.join().unwrap();
        position.current_price = current_price;
        position.gain = current_price * position.quantity as f64 - position.cost_basis;

        Ok(position)
    }

    pub fn calculate_all(db: &mongodb::db::Database) -> WalletResult<Vec<Position>> {
        let positions = Mutex::new(Vec::<Position>::new());

        let symbols = get_distinct_symbols(db)?;
        symbols
            .into_par_iter()
            .try_for_each::<_, WalletResult<_>>(|symbol| {
                let position = Position::calculate_for_symbol(db, &symbol)?;
                positions.lock().unwrap().push(position);
                Ok(())
            })?;

        Ok(positions.into_inner().unwrap())
    }

    pub fn save_snapshots(
        db: &mongodb::db::Database,
        symbol: &str,
        mut snapshots: Vec<Position>,
    ) -> WalletResult<()> {
        info!("[{}] saving Position snapshots", symbol);

        // Ensure we do not try to calculate for the same symbol more than once at a time.
        let _guard = LockMap::lock(Position::collection_name(), symbol);

        let historical = db.collection("historical");
        let find_options = Some(FindOptions::new()).map(|mut options| {
            options.sort = Some(doc! { "time": -1 });
            options
        });

        let mut previous_date: Option<DateTime<Local>> = None;
        for position in &mut snapshots {
            if let Some(previous_date) = previous_date {
                info!(
                    "[{}] generating snapshots for range {:?} -> {:?}",
                    symbol, previous_date, position.time
                );
                for friday in find_all_fridays_between(previous_date, position.time) {
                    // We search for historical prices over a week to make sure we get
                    // data even through weekends and holidays.
                    // FIXME: this version of the mongodb driver doesn't seem to like
                    // DateTime<Utc> objects. Newer ones work, maybe bite the bullet here.
                    let range_from = (friday - Duration::days(7)).and_hms(0, 0, 0).to_rfc3339();
                    let range_to = friday.and_hms(23, 59, 59).to_rfc3339();

                    let filter = doc! {
                        "$and": [
                            { "symbol": symbol.to_string() },
                            { "time": { "$gte": range_from } },
                            { "time": { "$lte": range_to } },
                        ]
                    };

                    let document = historical
                        .find_one(Some(filter), find_options.clone())
                        .map_err(|e| dang!(Database, e))?;

                    if let Some(document) = document {
                        let asset_day: AssetDay = bson::from_bson(Bson::Document(document))
                            .map_err(|e| dang!(Bson, e))?;

                        position.time = asset_day.time.with_timezone(&Local);
                        position.current_price = asset_day.close;
                    } else {
                        warn!(
                            "failed to find historical data for {} on {}",
                            symbol, friday
                        );
                        position.time = friday.with_timezone(&Local).and_hms(12, 0, 0);
                    }

                    position.gain =
                        position.current_price * position.quantity as f64 - position.cost_basis;

                    debug!("[{}] inserting snapshot {:?}", symbol, position);
                    insert_one(&db, position.clone())?;
                }
            }

            previous_date = Some(position.time);
        }
        info!("[{}] done saving Position snapshots", symbol);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::vec::Vec;

    use super::*;
    use crate::operation::{AssetKind, OperationKind};
    use crate::stock::StockOperation;

    #[test]
    fn position_calculation() {
        let db = WalletDB::get_connection();

        assert!(
            db.collection(BaseOperation::collection_name())
                .delete_many(doc! {}, None)
                .is_ok(),
            true
        );

        let mut stock = StockOperation {
            asset_kind: AssetKind::Stock,
            operation: BaseOperation {
                id: None,
                kind: OperationKind::Purchase,
                broker: String::from("FakeTestBroker"),
                portfolio: String::from("FakeTestWallet"),
                symbol: String::from("FAKE4"),
                time: Local::now(),
                price: 10.0,
                quantity: 100,
                fees: 0.0,
            },
        };

        let mut sales = Vec::<Sale>::new();

        assert!(insert_one(&db, stock.clone()).is_ok(), true);

        stock.operation.price = 12.0;
        stock.operation.quantity = 50;
        stock.operation.kind = OperationKind::Sale;
        stock.operation.time = Local::now();

        sales.push(Sale {
            time: stock.operation.time,
            quantity: 50,
            cost_price: 10.0,
            sell_price: 12.0,
        });

        assert!(insert_one(&db, stock.clone()).is_ok(), true);

        stock.operation.price = 4.0;
        stock.operation.kind = OperationKind::Purchase;
        stock.operation.time = Local::now();

        assert!(insert_one(&db, stock.clone()).is_ok(), true);

        let position = Position::calculate_for_symbol(&db, "FAKE4");
        assert_eq!(position.is_ok(), true);

        // FIXME: these values change dynamically, and return NaN with the fake ticker;
        // figure out how to test.
        let mut position = position.unwrap();
        position.current_price = 0.0;
        position.gain = 0.0;

        assert_eq!(
            position,
            Position {
                symbol: String::from("FAKE4"),
                average_price: 7.0,
                cost_basis: 700.0,
                quantity: 100,
                time: stock.operation.time,
                current_price: 0.0,
                gain: 0.0,
                realized: 600.0,
                sales,
            }
        );
    }

    #[test]
    fn test_calculate_all() {
        let db = WalletDB::get_connection();

        Position::calculate_all(&db).expect("Something went wrong");
    }
}
