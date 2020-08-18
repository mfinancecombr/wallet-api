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

    pub symbol: String,
    pub price: f64,
    pub quantity: i64,

    #[serde(default)]
    pub fees: f64,

    #[serde(default = "Local::now")]
    pub time: DateTime<Local>,

    #[serde(rename="type")]
    pub kind: OperationKind,

    #[serde(default = "default_broker")]
    pub broker: String,

    #[serde(default = "default_portfolio")]
    pub portfolio: String,
}

impl<'de> Queryable<'de> for BaseOperation {
    fn collection_name() -> &'static str { "operations" }
}

fn default_broker() -> String {
    "default".to_string()
}

fn default_portfolio() -> String {
    "default".to_string()
}