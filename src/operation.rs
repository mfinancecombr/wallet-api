use rocket_okapi::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::error::{BackendError, WalletResult};
use crate::walletdb::{Queryable, WalletDB};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum AssetKind {
    Stock,
    TesouroDireto,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum OperationKind {
    Purchase,
    Sale,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BaseOperation {
    pub price: f64,
    pub quantity: i64,

    #[serde(default)]
    pub fees: f64,

    #[serde(rename = "type")]
    pub kind: OperationKind,

    pub broker: Option<String>,

    #[serde(default = "Vec::<String>::new")]
    pub portfolios: Vec<String>,
}

impl Queryable for BaseOperation {
    fn collection_name() -> &'static str {
        "operations"
    }
}

pub fn get_distinct_symbols() -> WalletResult<Vec<String>> {
    let db = WalletDB::get_connection();
    let collection = db.collection("operations");

    let symbols = collection.distinct("symbol", None, None)?;

    symbols
        .iter()
        .map(|s| {
            s.as_str()
                .ok_or_else(|| dang!(Bson, "Failure converting string (symbol)"))
                .map(|s| s.to_string())
        })
        .collect::<WalletResult<Vec<String>>>()
}
