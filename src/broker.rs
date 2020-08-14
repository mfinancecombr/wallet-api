use std::vec;
use rocket_contrib::json::Json;
use rocket_okapi::{openapi, JsonSchema};
use serde::{Deserialize, Serialize};

use crate::error::{BackendError};
use crate::walletdb::*;


#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Broker {
    #[serde(alias = "_id")]
    id: String,
    name: String,
    cnpj: Option<String>,
}

impl<'de> Queryable<'de> for Broker {
    fn collection_name() ->&'static str { "brokers" }
}

/// # Add a broker
///
/// Adds a new broker
#[openapi]
#[post("/brokers", data = "<broker>")]
pub fn add_broker(db: WalletDB, broker: Json<Broker>) -> Result<(), BackendError> {
    insert_one::<Broker>(&*db, broker.into_inner())
}

/// # List brokers
///
/// Lists all brokers
#[openapi]
#[get("/brokers")]
pub fn get_brokers(db: WalletDB) -> Result<Json<Vec<Broker>>, BackendError> {
    get::<Broker>(&*db).map(|results| Json(results))
}

/// # Get broker
///
/// Get a specific broker
#[openapi]
#[get("/brokers/<oid>")]
pub fn get_broker_by_oid(db: WalletDB, oid: String) -> Result<Json<Broker>, BackendError> {
    get_one::<Broker>(&*db, oid).map(|results| Json(results))
}

/// # Update a broker
///
/// Update a specific broker
#[openapi]
#[put("/brokers/<oid>", data = "<broker>")]
pub fn update_broker_by_oid(db: WalletDB, oid: String, broker: Json<Broker>) -> Result<(), BackendError> {
    update_one::<Broker>(&*db, oid, broker.into_inner())
}

/// # Delete a broker
///
/// Delete a specific broker
#[openapi]
#[delete("/brokers/<oid>")]
pub fn delete_broker_by_oid(db: WalletDB, oid: String) -> Result<(), BackendError> {
    delete_one::<Broker>(&*db, oid)
}