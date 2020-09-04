use rocket::request::Form;
use rocket_contrib::json::Json;
use rocket_okapi::{openapi, JsonSchema};
use serde::{Deserialize, Serialize};

use crate::error::WalletResult;
use crate::rest::*;
use crate::walletdb::*;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Broker {
    #[serde(alias = "_id")]
    id: Option<String>,
    name: String,
    cnpj: Option<String>,
}

impl<'de> Queryable<'de> for Broker {
    fn collection_name() -> &'static str {
        "brokers"
    }
}

/// # Add a broker
///
/// Adds a new broker
#[openapi]
#[post("/brokers", data = "<broker>")]
pub fn add_broker(db: WalletDB, broker: Json<Broker>) -> WalletResult<Json<Broker>> {
    api_add(db, broker)
}

/// # List brokers
///
/// Lists all brokers
#[openapi]
#[get("/brokers?<options..>")]
pub fn get_brokers(
    db: WalletDB,
    options: Option<Form<ListingOptions>>,
) -> WalletResult<Rest<Json<Vec<Broker>>>> {
    api_get::<Broker>(db, options)
}

/// # Get broker
///
/// Get a specific broker
#[openapi]
#[get("/brokers/<oid>")]
pub fn get_broker_by_oid(db: WalletDB, oid: String) -> WalletResult<Json<Broker>> {
    api_get_one::<Broker>(db, oid)
}

/// # Update a broker
///
/// Update a specific broker
#[openapi]
#[put("/brokers/<oid>", data = "<broker>")]
pub fn update_broker_by_oid(
    db: WalletDB,
    oid: String,
    broker: Json<Broker>,
) -> WalletResult<Json<Broker>> {
    api_update::<Broker>(db, oid, broker)
}

/// # Delete a broker
///
/// Delete a specific broker
#[openapi]
#[delete("/brokers/<oid>")]
pub fn delete_broker_by_oid(db: WalletDB, oid: String) -> WalletResult<Json<Broker>> {
    api_delete::<Broker>(db, oid)
}
