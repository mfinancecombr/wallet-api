use rocket_contrib::json::Json;
use rocket_okapi::{openapi};

use crate::error::{WalletResult};
use crate::position::Position;
use crate::walletdb::WalletDB;


#[openapi]
#[post("/portfolio/position")]
pub fn portfolio_position(db: WalletDB) -> WalletResult<Json<Vec<Position>>> {
    Position::calculate_all(&*db).map(|result| Json(result))
}
