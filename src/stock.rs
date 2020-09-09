use rocket_contrib::json::Json;
use rocket_okapi::{openapi, JsonSchema};
use serde::{Deserialize, Serialize};

use crate::error::WalletResult;
use crate::operation::{AssetKind, BaseOperation};
use crate::position::Position;

fn asset_kind() -> AssetKind {
    AssetKind::Stock
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct StockOperation {
    #[serde(rename = "assetType", default = "asset_kind")]
    pub asset_kind: AssetKind,

    #[serde(flatten)]
    pub operation: BaseOperation,
}

/// # Get a stock position
///
/// Get position for a specific stock
#[openapi]
#[get("/stocks/position/<symbol>")]
pub fn get_stock_position_by_symbol(symbol: String) -> WalletResult<Json<Position>> {
    Position::calculate_for_symbol(&symbol).map(Json)
}
