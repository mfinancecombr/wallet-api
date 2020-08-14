use std::vec;
use rocket_contrib::json::Json;
use rocket_okapi::{openapi, JsonSchema};
use serde::{Deserialize, Serialize};

use crate::error::{BackendError};
use crate::operation::{BaseOperation, AssetKind};
use crate::walletdb::*;


fn asset_kind() -> AssetKind { AssetKind::Stock }

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct StockOperation {
    #[serde(rename = "assetType", default = "asset_kind")]
    pub asset_kind: AssetKind,

    #[serde(flatten)]
    pub operation: BaseOperation,
}

impl<'de> Queryable<'de> for StockOperation {
    fn collection_name() -> &'static str { "operations" }
}

/// # Add a stock operation
///
/// Adds a new stock operation
#[openapi]
#[post("/stocks/operations", data = "<operation>")]
pub fn add_stock_operation(db: WalletDB, operation: Json<StockOperation>) -> Result<(), BackendError> {
    insert_one::<StockOperation>(&*db, operation.into_inner())
}

/// # List stock operations
///
/// Lists all stock operations
#[openapi]
#[get("/stocks/operations")]
pub fn get_stock_operations(db: WalletDB) -> Result<Json<Vec<StockOperation>>, BackendError> {
    get::<StockOperation>(&*db).map(|results| Json(results))
}

/// # Get stock operation
///
/// Get a specific stock operation
#[openapi]
#[get("/stocks/operations/<oid>")]
pub fn get_stock_operation_by_oid(db: WalletDB, oid: String) -> Result<Json<StockOperation>, BackendError> {
    get_one::<StockOperation>(&*db, oid).map(|results| Json(results))
}

/// # Update a stock operation
///
/// Update a specific stock operation
#[openapi]
#[put("/stocks/operations/<oid>", data = "<operation>")]
pub fn update_stock_operation_by_oid(db: WalletDB, oid: String, operation: Json<StockOperation>) -> Result<(), BackendError> {
    update_one::<StockOperation>(&*db, oid, operation.into_inner())
}

/// # Delete a stock operation
///
/// Delete a specific stock operation
#[openapi]
#[delete("/stocks/operations/<oid>")]
pub fn delete_stock_operation_by_oid(db: WalletDB, oid: String) -> Result<(), BackendError> {
    delete_one::<StockOperation>(&*db, oid)
}
