use chrono::{DateTime, Utc};
use mongodb::bson::doc;
use rocket::request::Form;
use rocket_contrib::json::Json;
use rocket_okapi::{openapi, JsonSchema};
use serde::{Deserialize, Serialize};

use crate::error::{BackendError, WalletResult};
use crate::fii::FIIOperation;
use crate::rest::*;
use crate::stock::{StockOperation, StockSplit};
use crate::walletdb::{Queryable, WalletDB};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct Event {
    #[serde(alias = "_id")]
    pub id: Option<String>,

    pub symbol: String,

    #[serde(default = "Utc::now")]
    pub time: DateTime<Utc>,

    #[serde(flatten)]
    pub detail: EventDetail,
}

impl Queryable for Event {
    fn collection_name() -> &'static str {
        "events"
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(tag = "eventType", content = "detail")]
pub enum EventDetail {
    #[serde(rename = "stock-operation")]
    StockOperation(StockOperation),

    #[serde(rename = "stock-split")]
    StockSplit(StockSplit),

    #[serde(rename = "fii-operation")]
    FIIOperation(FIIOperation),
}

/// # Add an event
///
/// Adds a new event
#[openapi]
#[post("/events", data = "<event>")]
pub fn add_event(event: Json<Event>) -> WalletResult<Json<Event>> {
    api_add::<Event>(event)
}

/// # List events
///
/// Lists all events
#[openapi]
#[get("/events?<options..>")]
pub fn get_events(options: Option<Form<ListingOptions>>) -> WalletResult<Rest<Json<Vec<Event>>>> {
    api_get::<Event>(None, options)
}

/// # Get event
///
/// Get a specific event
#[openapi]
#[get("/events/<oid>")]
pub fn get_event_by_oid(oid: String) -> WalletResult<Json<Event>> {
    api_get_one::<Event>(oid)
}

/// # Update an event
///
/// Update a specific event
#[openapi]
#[put("/events/<oid>", data = "<event>")]
pub fn update_event_by_oid(oid: String, event: Json<Event>) -> WalletResult<Json<Event>> {
    api_update::<Event>(oid, event)
}

/// # Delete an event
///
/// Delete a specific event
#[openapi]
#[delete("/events/<oid>")]
pub fn delete_event_by_oid(oid: String) -> WalletResult<Json<Event>> {
    api_delete::<Event>(oid)
}

pub fn get_distinct_symbols(oid: Option<String>) -> WalletResult<Vec<String>> {
    let db = WalletDB::get_connection();
    let collection = db.collection("events");

    let filter = oid.map(|oid| {
        doc! {
            "detail.portfolios": &oid
        }
    });

    let symbols = collection.distinct("symbol", filter, None)?;

    symbols
        .iter()
        .map(|s| {
            s.as_str()
                .ok_or_else(|| dang!(Bson, "Failure converting string (symbol)"))
                .map(|s| s.to_string())
        })
        .collect::<WalletResult<Vec<String>>>()
}
