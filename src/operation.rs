use std::fmt::{Debug};
use chrono::{Local, DateTime};
use rocket_okapi::{JsonSchema};
use serde::{Deserialize, Serialize};

use crate::walletdb::Queryable;


#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="lowercase")]
pub enum AssetKind {
    Stock,
    TesouroDireto
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="lowercase")]
pub enum OperationKind {
    Purchase,
    Sale
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BaseOperation {
    #[serde(alias = "_id")]
    pub id: Option<String>,

    #[serde(rename="type")]
    pub kind: OperationKind,

    pub broker: String,
    pub portfolio: String,

    pub symbol: String,
    pub time: DateTime<Local>,
    pub price: f64,
    pub quantity: i64,
    pub fees: f64,
}

impl<'de> Queryable<'de> for BaseOperation {
    fn collection_name() -> &'static str { "operations" }
}
