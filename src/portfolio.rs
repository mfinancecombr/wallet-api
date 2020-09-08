use rocket_contrib::json::Json;
use rocket_okapi::openapi;

use crate::error::WalletResult;
use crate::position::Position;

#[openapi]
#[post("/portfolio/position")]
pub fn portfolio_position() -> WalletResult<Json<Vec<Position>>> {
    Position::calculate_all().map(Json)
}
