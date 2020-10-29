use mongodb::bson::doc;
use rocket_okapi::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::walletdb::Queryable;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum AssetKind {
    Stock,
    TesouroDireto,
    FII,
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
