use chrono::{Date, DateTime, Datelike, Duration, TimeZone, Utc, Weekday};
use log::{debug, info, warn};
use mongodb::bson::{doc, Bson};
use mongodb::options::{FindOneOptions, FindOptions};
use rayon::prelude::*;
use rocket_okapi::JsonSchema;
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::error::*;
use crate::event::{get_distinct_symbols, Event, EventDetail};
use crate::historical::Historical;
use crate::operation::{BaseOperation, OperationKind};
use crate::scheduling::LockMap;
use crate::stock::StockSplitKind;
use crate::walletdb::*;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    #[serde(alias = "_id")]
    pub id: Option<String>,
    pub symbol: String,
    pub average_price: f64,
    pub cost_basis: f64,
    pub quantity: i64,
    pub time: DateTime<Utc>,
    pub current_price: f64,
    pub gain: f64,
    pub realized: f64,
    pub recent_operations: Vec<BaseOperation>,
    pub portfolio: Option<String>,
}

impl Position {
    fn new(symbol: &str, portfolio_oid: Option<String>) -> Self {
        Position {
            id: None,
            symbol: symbol.to_string(),
            cost_basis: 0.0,
            quantity: 0,
            average_price: 0.0,
            time: Utc::now(),
            current_price: 0.0,
            gain: 0.0,
            realized: 0.0,
            recent_operations: Vec::<BaseOperation>::new(),
            portfolio: portfolio_oid,
        }
    }

    pub fn cmp_symbol(a: &Position, b: &Position) -> std::cmp::Ordering {
        a.symbol.cmp(&b.symbol)
    }

    pub fn cmp_id(a: &Position, b: &Position) -> std::cmp::Ordering {
        a.id.cmp(&b.id)
    }

    pub fn cmp_quantity(a: &Position, b: &Position) -> std::cmp::Ordering {
        a.quantity.partial_cmp(&b.quantity).unwrap()
    }

    pub fn float_cmp(a: &f64, b: &f64) -> std::cmp::Ordering {
        match (a.is_nan(), b.is_nan()) {
            (true, true) => std::cmp::Ordering::Equal,
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            (false, false) => a.partial_cmp(&b).unwrap(),
        }
    }

    pub fn cmp_average_price(a: &Position, b: &Position) -> std::cmp::Ordering {
        Position::float_cmp(&a.average_price, &b.average_price)
    }

    pub fn cmp_current_price(a: &Position, b: &Position) -> std::cmp::Ordering {
        Position::float_cmp(&a.current_price, &b.current_price)
    }

    pub fn cmp_cost_basis(a: &Position, b: &Position) -> std::cmp::Ordering {
        Position::float_cmp(&a.cost_basis, &b.cost_basis)
    }

    pub fn cmp_current_value(a: &Position, b: &Position) -> std::cmp::Ordering {
        Position::float_cmp(
            &(a.current_price * a.quantity as f64),
            &(b.current_price * b.quantity as f64),
        )
    }

    pub fn cmp_gain(a: &Position, b: &Position) -> std::cmp::Ordering {
        // The web UI shows gain as a percentage.
        Position::float_cmp(&(a.gain / a.cost_basis), &(b.gain / b.cost_basis))
    }
}

impl Queryable for Position {
    fn collection_name() -> &'static str {
        "positions"
    }
}

fn find_all_fridays_between(from: DateTime<Utc>, to: DateTime<Utc>) -> Vec<Date<Utc>> {
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
async fn do_calculate_for_symbol(
    symbol: String,
    portfolio_oid: Option<String>,
) -> WalletResult<Position> {
    // Ensure we do not try to calculate for the same symbol more than once at a time.
    // Create it here so it is locked even before the thread gets to run, to avoid
    // races with callers of this function or multiple calls of this function.
    let guard = LockMap::lock(Position::collection_name(), &symbol);

    let db = WalletDB::get_connection();
    let collection = db.collection(Event::collection_name());

    let mut date_from = Utc.timestamp(61, 0);

    // If we already have a bunch of position snapshots, we pick up
    // from the last one rather than starting from scratch.
    let mut position = Position::last(&symbol, portfolio_oid.clone())
        .map(|pos| {
            date_from = pos.time.with_timezone(&Utc);
            pos
        })
        .unwrap_or_else(|| Position::new(&symbol, portfolio_oid.clone()));
    let mut filter = doc! {
        "$and": [
            { "symbol": &symbol },
            {
                "time": {
                    "$lte": Utc::today().and_hms(23, 59, 59).to_rfc3339()
                }
            },
            {
                "time": {
                    "$gt": date_from.to_rfc3339()
                }
            }
        ]
    };

    if let Some(portfolio_oid) = portfolio_oid {
        filter
            .get_array_mut("$and")
            .unwrap()
            .push(Bson::Document(doc! {
                "detail.portfolios": portfolio_oid
            }));
    }

    let options = FindOptions::builder().sort(doc! { "time": 1 });
    let cursor = collection.find(filter, options.build())?;

    let mut references = Vec::<Position>::new();
    for document in cursor {
        if let Ok(document) = document {
            let event = Event::from_doc(document)?;

            position.time = event.time;

            match event.detail {
                EventDetail::StockOperation(operation) => {
                    let operation = operation.operation;
                    match operation.kind {
                        OperationKind::Purchase => {
                            position.cost_basis += operation.price * operation.quantity as f64;
                            position.quantity += operation.quantity;
                        }
                        OperationKind::Sale => {
                            /* When selling we need to use the average price at the moment
                             * of the sale for the average calculation to work. We may
                             * take out too little if the current price is lower or too
                             * much, otherwise.
                             */
                            let cost_price = position.cost_basis / position.quantity as f64;
                            position.cost_basis -= cost_price * operation.quantity as f64;
                            position.quantity -= operation.quantity;

                            position.realized += operation.quantity as f64 * operation.price
                                - operation.quantity as f64 * cost_price;
                        }
                    }

                    if position.quantity != 0 && position.cost_basis != 0.0 {
                        position.average_price = position.cost_basis / position.quantity as f64;
                    }

                    position.recent_operations.push(operation.clone());
                }
                EventDetail::StockSplit(split) => match split.split_kind {
                    StockSplitKind::Split => {
                        position.quantity *= split.factor;
                        position.average_price /= split.factor as f64;
                    }
                    StockSplitKind::ReverseSplit => {
                        position.quantity /= split.factor;
                        position.average_price *= split.factor as f64;
                    }
                },
            }

            references.push(position.clone());
            position.recent_operations.clear();
        }
    }

    // Up to here we used the time for the last operation, but we have been asked
    // for the "current" position. We also need to add that to references' last position,
    // so that the snapshots will be calculated up to today.
    position.time = Utc::now();
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
    pub fn last(symbol: &str, portfolio_oid: Option<String>) -> Option<Self> {
        let db = WalletDB::get_connection();
        let collection = db.collection(Position::collection_name());

        let filter = if let Some(portfolio_oid) = portfolio_oid {
            doc! {
                "$and": [
                    { "symbol": symbol.to_string() },
                    { "portfolio": portfolio_oid }
                ]
            }
        } else {
            doc! { "symbol": symbol.to_string() }
        };

        let options = FindOneOptions::builder().sort(doc! { "time": -1 }).build();

        if let Ok(doc) = collection.find_one(filter, options) {
            if let Some(doc) = doc {
                Position::from_doc(doc).ok()
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn calculate_for_symbol(
        symbol: &str,
        portfolio_oid: Option<String>,
    ) -> WalletResult<Position> {
        // Ensure we do not try to calculate for the same symbol more than once at a time.
        let _guard = LockMap::lock(Event::collection_name(), symbol);

        // Fire a background thread to get the current price.
        let ysymbol = symbol.to_string();
        let current_price =
            std::thread::spawn(move || Historical::current_price_for_symbol(ysymbol));

        let symbol = symbol.to_string();
        let mut position =
            std::thread::spawn(move || do_calculate_for_symbol(symbol, portfolio_oid))
                .join()
                .unwrap()?;

        // We only care about current price if we still have a position. If not, let's skip this step.
        if position.quantity > 0 {
            let current_price = current_price.join().unwrap();
            position.current_price = current_price;
            position.gain = current_price * position.quantity as f64 - position.cost_basis;
        }

        Ok(position)
    }

    pub fn calculate_all() -> WalletResult<Vec<Position>> {
        Position::get_all_for_portfolio(None)
    }

    pub fn get_all_for_portfolio(oid: Option<String>) -> WalletResult<Vec<Position>> {
        let positions = Mutex::new(Vec::<Position>::new());

        let symbols = get_distinct_symbols(oid.clone())?;
        symbols
            .into_par_iter()
            .try_for_each::<_, WalletResult<_>>(|symbol| {
                let position = Position::calculate_for_symbol(&symbol, oid.clone())?;

                // Old positions will show up here. Maybe we will want to include them
                // for future views and need a parameter for this function, but for now
                // just ignore them.
                if position.quantity > 0 {
                    positions.lock().unwrap().push(position);
                }

                Ok(())
            })?;

        let mut positions = positions.into_inner().unwrap();

        // The react-admin query interface expects to find ids, but we did not
        // necessarily get these from the database. So we make up fake ids.
        for (count, position) in positions.iter_mut().enumerate() {
            position.id = Some(count.to_string());
        }

        positions.sort_unstable_by(|a, b| a.symbol.partial_cmp(&b.symbol).unwrap());

        Ok(positions)
    }

    pub fn get_history_for_portfolio(
        oid: Option<String>,
        since: Option<DateTime<Utc>>,
    ) -> WalletResult<HashMap<Date<Utc>, Vec<Position>>> {
        let db = WalletDB::get_connection();
        let collection = db.collection(Position::collection_name());

        let since = since.unwrap_or_else(|| Utc.ymd(2006, 1, 1).and_hms(0, 0, 0));
        let filter = if let Some(oid) = oid {
            doc! {
                "portfolio": oid,
                "time": { "$gt": since.to_rfc3339() }
            }
        } else {
            doc! {
                "time": { "$gt": since.to_rfc3339() }
            }
        };

        let options = FindOptions::builder().sort(doc! { "time": 1 });

        let positions = collection
            .find(filter, options.build())
            .map(|cursor| Position::from_docs(cursor).expect("Failed to convert document"))
            .expect("Failed to query positions collection");

        let mut snapshots = HashMap::<Date<Utc>, Vec<Position>>::new();

        for position in positions {
            snapshots
                .entry(position.time.date())
                .or_insert(vec![])
                .push(position);
        }

        Ok(snapshots)
    }

    pub fn create_snapshots(symbol: &str, mut references: Vec<Position>) -> WalletResult<()> {
        info_!("[{}] saving Position snapshots", symbol);

        let mut previous_position: Option<Position> = None;
        for position in &mut references {
            if let Some(mut previous_position) = previous_position {
                info_!(
                    "[{}] generating snapshots for range {:?} -> {:?}",
                    symbol,
                    previous_position.time,
                    position.time
                );
                for friday in find_all_fridays_between(previous_position.time, position.time) {
                    let asset_day = Historical::get_for_day_with_fallback(symbol, friday);
                    if let Ok(asset_day) = asset_day {
                        previous_position.time = friday.and_hms(12, 0, 0);
                        previous_position.current_price = asset_day.close;
                    } else {
                        warn!(
                            "failed to find historical data for {} on {}",
                            symbol, friday
                        );
                        previous_position.time = friday.and_hms(12, 0, 0);
                    }

                    previous_position.gain = previous_position.current_price
                        * previous_position.quantity as f64
                        - previous_position.cost_basis;

                    debug!("[{}] inserting snapshot {:?}", symbol, previous_position);
                    insert_one(previous_position.clone())?;

                    previous_position.recent_operations.clear();
                }
            }

            previous_position = Some(position.clone());
        }

        // Make snapshots come up to yesterday.
        if let Some(mut previous_position) = previous_position {
            for friday in
                find_all_fridays_between(previous_position.time, Utc::now() - Duration::days(1))
            {
                previous_position.time = friday.and_hms(12, 0, 0);
                debug!("[{}] inserting snapshot {:?}", symbol, previous_position);
                insert_one(previous_position.clone())?;
                previous_position.recent_operations.clear();
            }
        }

        info_!("[{}] done saving Position snapshots", symbol);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use rusty_fork::rusty_fork_test;
    use std::vec::Vec;

    use super::*;
    use crate::operation::{AssetKind, BaseOperation, OperationKind};
    use crate::portfolio::Portfolio;
    use crate::stock::{StockOperation, StockSplit};

    rusty_fork_test! {
        #[test]
        fn position_calculation() {
            WalletDB::init_client("mongodb://localhost:27017/");

            let db = WalletDB::get_connection();

            assert!(
                db.collection(Event::collection_name())
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

            let default_operation = EventDetail::StockOperation(StockOperation {
                asset_kind: AssetKind::Stock,
                operation: BaseOperation {
                    kind: OperationKind::Purchase,
                    broker: None,
                    portfolios: Vec::<String>::new(),
                    price: 10.0,
                    quantity: 100,
                    fees: 0.0,
                },
            });

            let mut event = Event {
                id: None,
                symbol: symbol.clone(),
                time: Utc.ymd(2020, 1, 1).and_hms(12, 0, 0),
                detail: default_operation.clone(),
            };

            let mut recent_operations = Vec::<BaseOperation>::new();

            assert!(insert_one(event.clone()).is_ok(), true);

            let mut detail = std::mem::replace(&mut event.detail, default_operation.clone());
            if let EventDetail::StockOperation(operation) = &mut detail {
                let operation = &mut operation.operation;
                event.time = Utc.ymd(2020, 2, 1).and_hms(12, 0, 0);
                operation.price = 12.0;
                operation.quantity = 50;
                operation.kind = OperationKind::Sale;

                recent_operations.push(operation.clone());

                event.detail = detail;

                assert!(insert_one(event.clone()).is_ok(), true);
            }

            let portfolio = insert_one(Portfolio {
                id: None,
                name: "FakePortfolio".to_string(),
            })
            .expect("Failed to insert Portfolio");

            let mut detail = std::mem::replace(&mut event.detail, default_operation.clone());
            if let EventDetail::StockOperation(operation) = &mut detail {
                let operation = &mut operation.operation;
                event.time = Utc.ymd(2020, 3, 1).and_hms(12, 0, 0);
                operation.price = 4.0;
                operation.kind = OperationKind::Purchase;
                operation
                    .portfolios
                    .push(portfolio.id.as_ref().unwrap().clone());

                event.detail = detail;

                assert!(insert_one(event.clone()).is_ok(), true);
            }

            let split = EventDetail::StockSplit(StockSplit {
                split_kind: StockSplitKind::Split,
                factor: 2,
            });

            let operation = std::mem::replace(&mut event.detail, split);

            event.time = Utc.ymd(2020, 3, 2).and_hms(12, 0, 0);
            assert!(insert_one(event.clone()).is_ok(), true);

            let _ = std::mem::replace(&mut event.detail, operation);

            let mut detail = std::mem::replace(&mut event.detail, default_operation);
            if let EventDetail::StockOperation(operation) = &mut detail {
                let operation = &mut operation.operation;
                // This is a Friday, so will test corner cases of the position snapshots.
                event.time = Utc.ymd(2020, 3, 27).and_hms(12, 0, 0);
                operation.price = 5.0;
                operation.quantity *= 2;
                operation.kind = OperationKind::Purchase;

                event.detail = detail;

                assert!(insert_one(event).is_ok(), true);
            }

            // Do a full update first, which should trigger calculation for our
            // FAKE4. This means the specific call below should start from an
            // existing reference.
            Position::calculate_all().expect("Something went wrong");

            let position = Position::calculate_for_symbol("FAKE4", None);
            assert_eq!(position.is_ok(), true);
            let position = position.unwrap();

            let same_position = Position::calculate_for_symbol("FAKE4", None);
            assert_eq!(same_position.is_ok(), true);
            let same_position = same_position.unwrap();

            assert_relative_eq!(position.cost_basis, same_position.cost_basis,);
            assert_eq!(position.quantity, same_position.quantity);
            assert_relative_eq!(position.average_price, same_position.average_price);
            assert_relative_eq!(position.realized, same_position.realized);
            assert_eq!(position.recent_operations, same_position.recent_operations);

            // Manually check that the time is pretty close to now, since we will update our
            // reference below with what we got.
            assert!(Utc::now() - position.time < Duration::seconds(10));

            // NOTE: Our Historical mock for now just returns a static 9.0 price for all requests.
            assert_eq!(
                position,
                Position {
                    id: position.id.clone(),
                    symbol,
                    average_price: 4.0,
                    cost_basis: 1200.0,
                    quantity: 300,
                    time: position.time,
                    current_price: 9.0,
                    gain: 1500.0,
                    realized: 100.0,
                    recent_operations: vec![],
                    portfolio: None,
                }
            );

            // Ensure create_snapshots finished.
            let guard = LockMap::lock(Position::collection_name(), "FAKE4");
            drop(guard);

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
                ("2020-03-06", 700.0, 200, 100.0, 1100.0),
                ("2020-03-13", 700.0, 200, 100.0, 1100.0),
                ("2020-03-20", 700.0, 200, 100.0, 1100.0),
                ("2020-03-27", 1200.0, 300, 100.0, 1500.0),
                ("2020-04-03", 1200.0, 300, 100.0, 1500.0),
            ];

            for (index, position) in positions.into_iter().enumerate() {
                let (time, cost_basis, quantity, realized, gain) = &expected[index];
                assert_eq!(*time, position.time.naive_local().date().to_string());
                assert_relative_eq!(*cost_basis, position.cost_basis);
                assert_eq!(*quantity, position.quantity);
                assert_relative_eq!(*realized, position.realized);
                assert_relative_eq!(*gain, position.gain);
            }

            let position = Position::calculate_for_symbol("FAKE4", portfolio.id.clone());
            assert_eq!(position.is_ok(), true);

            // Wait for create_snapshots to finish.
            let guard = LockMap::lock(Position::collection_name(), "FAKE4");
            drop(guard);

            // Make sure snapshots were created for the portfolio as well.
            let filter = doc! {
                "$and": [
                    { "time": { "$lt": "2020-04-04" } },
                    { "portfolio": portfolio.id.unwrap() }
                ]
            };

            let positions = collection
                .find(Some(filter), None)
                .map(|cursor| Position::from_docs(cursor).expect("Failed to convert document"))
                .expect("Failed to query positions collection");

            // This portfolio should have fewer entries, since its first operation
            // is from March 1st.
            assert_eq!(positions.len(), 5);

            if let Err(e) = db.drop(None) {
                println!("Failed to drop test db {}", format!("{:?}", e));
            }
        }
    }
}
