use std::cmp::PartialEq;
use chrono::{DateTime, Date, Local, Utc};
use mongodb::{bson, doc};
use mongodb::db::ThreadedDatabase;
use rocket_okapi::{JsonSchema};
use serde::{Serialize, Deserialize};

use crate::error::*;
use crate::operation::{BaseOperation, OperationKind};
use crate::walletdb::Queryable;


#[derive(Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct Position {
    pub symbol: String,
    pub average_price: f64,
    pub cost_basis: f64,
    pub quantity: i64,
    pub date: DateTime<Local>,
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

        let cursor = match collection.find(Some(filter), None) {
            Ok(cursor) => cursor,
            Err(e) => {
                return Err(dang!(Database, e));
            }
        };

        let mut total_cost = 0f64;
        let mut total_quantity = 0i64;

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
                        let price = total_cost / total_quantity as f64;
                        total_cost -= price * quantity as f64;
                        total_quantity -= quantity;
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

        Ok(Position {
            symbol: symbol.to_string(),
            cost_basis: total_cost,
            quantity: total_quantity,
            average_price: average,
            date: date_to
        })
    }
}

#[cfg(test)]
mod tests {
    use mongodb::ThreadedClient;

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

        assert!(insert_one(&db, stock.clone()).is_ok(), true);

        stock.operation.price = 12.0;
        stock.operation.quantity = 50;
        stock.operation.kind = OperationKind::Sale;
        stock.operation.time = Local::now();

        assert!(insert_one(&db, stock.clone()).is_ok(), true);

        stock.operation.price = 4.0;
        stock.operation.kind = OperationKind::Purchase;
        stock.operation.time = Local::now();

        assert!(insert_one(&db, stock.clone()).is_ok(), true);

        let date_to = Local::today();
        let position = Position::calculate_for_symbol(&db, "FAKE4", Some(date_to));
        assert_eq!(position.is_ok(), true);
        assert_eq!(
            position.unwrap(),
            Position {
                symbol: String::from("FAKE4"),
                average_price: 7.0,
                cost_basis: 700.0,
                quantity: 100,
                date: date_to.and_hms(23, 59, 59),
            }
        );
    }
}