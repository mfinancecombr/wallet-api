use rocket_contrib::json::Json;
use rocket_okapi::{openapi, JsonSchema};
use serde::{Deserialize, Serialize};

use crate::error::WalletResult;
use crate::operation::{AssetKind, BaseOperation};
use crate::position::Position;

fn asset_kind() -> AssetKind {
    AssetKind::FII
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct FIIOperation {
    #[serde(rename = "assetType", default = "asset_kind")]
    pub asset_kind: AssetKind,

    #[serde(flatten)]
    pub operation: BaseOperation,
}

/// # Get a FII position
///
/// Get FII for a specific stock
#[openapi]
#[get("/fiis/position/<symbol>")]
pub fn get_fii_position_by_symbol(symbol: String) -> WalletResult<Json<Position>> {
    Position::calculate_for_symbol(&symbol, None).map(Json)
}
