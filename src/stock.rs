use rocket::request::Form;
use rocket_contrib::json::Json;
use rocket_okapi::{openapi, JsonSchema};
use serde::{Deserialize, Serialize};

use crate::error::WalletResult;
use crate::operation::{AssetKind, BaseOperation};
use crate::position::Position;
use crate::rest::*;
use crate::walletdb::*;

fn asset_kind() -> AssetKind {
    AssetKind::Stock
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct StockOperation {
    #[serde(rename = "assetType", default = "asset_kind")]
    pub asset_kind: AssetKind,

    #[serde(flatten)]
    pub operation: BaseOperation,
}

impl Queryable for StockOperation {
    fn collection_name() -> &'static str {
        "operations"
    }
}

/// # Add a stock operation
///
/// Adds a new stock operation
#[openapi]
#[post("/stocks/operations", data = "<operation>")]
pub fn add_stock_operation(operation: Json<StockOperation>) -> WalletResult<Json<StockOperation>> {
    api_add::<StockOperation>(operation)
}

/// # List stock operations
///
/// Lists all stock operations
#[openapi]
#[get("/stocks/operations?<options..>")]
pub fn get_stock_operations(
    options: Option<Form<ListingOptions>>,
) -> WalletResult<Rest<Json<Vec<StockOperation>>>> {
    api_get::<StockOperation>(None, options)
}

/// # Get stock operation
///
/// Get a specific stock operation
#[openapi]
#[get("/stocks/operations/<oid>")]
pub fn get_stock_operation_by_oid(oid: String) -> WalletResult<Json<StockOperation>> {
    api_get_one::<StockOperation>(oid)
}

/// # Update a stock operation
///
/// Update a specific stock operation
#[openapi]
#[put("/stocks/operations/<oid>", data = "<operation>")]
pub fn update_stock_operation_by_oid(
    oid: String,
    operation: Json<StockOperation>,
) -> WalletResult<Json<StockOperation>> {
    api_update::<StockOperation>(oid, operation)
}

/// # Delete a stock operation
///
/// Delete a specific stock operation
#[openapi]
#[delete("/stocks/operations/<oid>")]
pub fn delete_stock_operation_by_oid(oid: String) -> WalletResult<Json<StockOperation>> {
    api_delete::<StockOperation>(oid)
}

/// # Get a stock position
///
/// Get position for a specific stock
#[openapi]
#[get("/stocks/position/<symbol>")]
pub fn get_stock_position_by_symbol(symbol: String) -> WalletResult<Json<Position>> {
    Position::calculate_for_symbol(&symbol).map(Json)
}
