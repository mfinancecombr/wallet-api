use chrono::{DateTime, Local};
use rocket_okapi::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::error::{BackendError, WalletResult};
use crate::walletdb::{Queryable, WalletDB};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum AssetKind {
    Stock,
    TesouroDireto,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum OperationKind {
    Purchase,
    Sale,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BaseOperation {
    #[serde(alias = "_id")]
    pub id: Option<String>,

    pub symbol: String,
    pub price: f64,
    pub quantity: i64,

    #[serde(default)]
    pub fees: f64,

    #[serde(default = "Local::now")]
    pub time: DateTime<Local>,

    #[serde(rename = "type")]
    pub kind: OperationKind,

    #[serde(default = "default_broker")]
    pub broker: String,

    #[serde(default = "default_portfolio")]
    pub portfolios: Vec<String>,
}

impl Queryable for BaseOperation {
    fn collection_name() -> &'static str {
        "operations"
    }
}

fn default_broker() -> String {
    "default".to_string()
}

fn default_portfolio() -> Vec<String> {
    vec!["default".to_string()]
}

pub fn get_distinct_symbols() -> WalletResult<Vec<String>> {
    let db = WalletDB::get_connection();
    let collection = db.collection("operations");

    let symbols = collection
        .distinct("symbol", None, None)
        .map_err(|e| dang!(Database, e))?;

    symbols
        .iter()
        .map(|s| {
            s.as_str()
                .ok_or_else(|| dang!(Bson, "Failure converting string (symbol)"))
                .map(|s| s.to_string())
        })
        .collect::<WalletResult<Vec<String>>>()
}
