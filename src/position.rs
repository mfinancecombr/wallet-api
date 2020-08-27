use std::cmp::PartialEq;
use std::sync::Mutex;
use chrono::{DateTime, Date, Local, Utc};
use mongodb::{bson, doc};
use mongodb::db::ThreadedDatabase;
use rayon::prelude::*;
use rocket_okapi::{JsonSchema};
use serde::{Serialize, Deserialize};
use yahoo_finance::{history};

use crate::error::*;
use crate::operation::{BaseOperation, OperationKind};
use crate::walletdb::Queryable;


#[derive(Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct Sale {
    pub time: DateTime<Local>,
    pub quantity: i64,
    pub cost_price: f64,
    pub sell_price: f64,
}

fn get_safely<'de, T>(doc: &bson::ordered::OrderedDocument, key: &str) -> WalletResult<T>
    where T: Deserialize<'de>
{
    if let Some(value) = doc.get(key) {
        bson::from_bson::<T>(value.clone()).map_err(|e| dang!(Bson, e))
    } else {
        Err(dang!(Database,
            format!("field `{}` not found on document", key)
        ))
    }
}

impl Position {
    pub fn calculate_for_symbol(
        db: &mongodb::db::Database,
        symbol: &str,
        date_to: Option<Date<Local>>
    ) -> WalletResult<Position>
    {
        let collection = db.collection(BaseOperation::collection_name());

        let date_to = date_to.unwrap_or(Local::today()).and_hms(23, 59, 59);
        let filter = doc!{
            "$and": [
                { "symbol": symbol },
                {
                    "time": {
                        "$lte": date_to.with_timezone(&Utc).to_rfc3339()
                    }
                }
            ]
        };

        // Fire a background thread to get the current price.
        let ysymbol= format!("{}.SA", &symbol);
        let current_price = std::thread::spawn(move || {
            let date_from = date_to.date().and_hms(0, 0, 0);
            let bar = history::retrieve_range(
                &ysymbol,
                DateTime::<Utc>::from(date_from),
                Some(DateTime::<Utc>::from(date_to)),
            ).ok().and_then(|mut bar| bar.pop());

            if let Some(bar) = bar {
                bar.close
            } else {
                f64::NAN
            }
        });

        let cursor = match collection.find(Some(filter), None) {
            Ok(cursor) => cursor,
            Err(e) => {
                return Err(dang!(Database, e));
            }
        };

        let mut total_cost = 0f64;
        let mut total_quantity = 0i64;
        let mut total_realized = 0f64;
        let mut sales = Vec::<Sale>::new();

        for document in cursor {
            if let Ok(document) = document {
                let quantity = get_safely::<i64>(&document, "quantity")?;
                let kind = get_safely::<OperationKind>(&document, "type")?;
                match kind {
                    OperationKind::Purchase => {
                        let price = get_safely::<f64>(&document, "price")?;
                        total_cost += price * quantity as f64;
                        total_quantity += quantity;
                    },
                    OperationKind::Sale => {
                        /* When selling we need to use the average price at the moment
                         * of the sale for the average calculation to work. We may
                         * take out too little if the current price is lower or too
                         * much, otherwise.
                         */
                        let cost_price = total_cost / total_quantity as f64;
                        total_cost -= cost_price * quantity as f64;
                        total_quantity -= quantity;

                        let sell_price = get_safely::<f64>(&document, "price")?;
                        total_realized += quantity as f64 * sell_price;

                        let time = get_safely::<DateTime<Utc>>(&document, "time")?;
                        sales.push(Sale {
                            time: DateTime::<Local>::from(time),
                            quantity: quantity,
                            cost_price: cost_price,
                            sell_price: sell_price,
                        })
                    },
                }
            }
        }

        let average;
        if total_quantity == 0 || total_cost == 0.0 {
            average = 0.0;
        } else {
            average = total_cost / total_quantity as f64;
        }

        // Get the result of our background thread. We just unwrap the
        // results as all errors are handled, so any panics should take
        // down execution anyway.
        let current_price = current_price.join().unwrap();
        Ok(Position {
            symbol: symbol.to_string(),
            cost_basis: total_cost,
            quantity: total_quantity,
            average_price: average,
            time: date_to,
            current_price: current_price,
            gain: current_price * total_quantity as f64 - total_cost,
            realized: total_realized,
            sales: sales,
        })
    }

    pub fn calculate_all(db: &mongodb::db::Database) -> WalletResult<Vec<Position>> {
        let collection = db.collection("operations");

        let symbols = collection.distinct("symbol", None, None)
            .map_err(|e| dang!(Database, e))?;

        let symbols = symbols.iter().map(|s|
            s.as_str()
                .ok_or(dang!(Bson, "Failure converting string (symbol)"))
        ).collect::<WalletResult<Vec<&str>>>()?;

        let positions = Mutex::new(Vec::<Position>::new());
        symbols.into_par_iter()
            .try_for_each::<_, WalletResult<_>>(|symbol| {
                let position = Position::calculate_for_symbol(db, symbol, None)?;
                positions.lock().unwrap().push(position);
                Ok(())
            }
        )?;

        Ok(positions.into_inner().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use mongodb::ThreadedClient;
    use std::vec::Vec;

    use super::*;
    use crate::operation::{AssetKind, OperationKind};
    use crate::stock::StockOperation;
    use crate::walletdb::*;

    #[test]
    fn position_calculation() {
        let db_client = mongodb::Client::with_uri("mongodb://127.0.0.1:27017/")
            .expect("Could not connect to mongodb");
        let db = db_client.db("finance-wallet-fake-test");

        assert!(db.collection(BaseOperation::collection_name()).delete_many(doc!{}, None).is_ok(), true);

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
            }
        };

        let mut sales = Vec::<Sale>::new();

        assert!(insert_one(&db, stock.clone()).is_ok(), true);

        stock.operation.price = 12.0;
        stock.operation.quantity = 50;
        stock.operation.kind = OperationKind::Sale;
        stock.operation.time = Local::now();

        sales.push(Sale {
            time: stock.operation.time.clone(),
            quantity: 50,
            cost_price: 10.0,
            sell_price: 12.0,
        });

        assert!(insert_one(&db, stock.clone()).is_ok(), true);

        stock.operation.price = 4.0;
        stock.operation.kind = OperationKind::Purchase;
        stock.operation.time = Local::now();

        assert!(insert_one(&db, stock.clone()).is_ok(), true);

        let date_to = Local::today();
        let position = Position::calculate_for_symbol(&db, "FAKE4", Some(date_to));
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
                time: date_to.and_hms(23, 59, 59),
                current_price: 0.0,
                gain: 0.0,
                realized: 600.0,
                sales: sales,
            }
        );
    }

    #[test]
    fn test_calculate_all() {
        let db_client = mongodb::Client::with_uri("mongodb://127.0.0.1:27017/")
            .expect("Could not connect to mongodb");
        let db = db_client.db("wallet-fake-test");

        Position::calculate_all(&db).expect("Something went wrong");
    }
}