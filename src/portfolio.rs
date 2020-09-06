use rocket::request::Form;
use rocket_contrib::json::Json;
use rocket_okapi::{openapi, JsonSchema};
use serde::{Deserialize, Serialize};

use crate::error::WalletResult;
use crate::position::Position;
use crate::rest::*;
use crate::walletdb::Queryable;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Portfolio {
    #[serde(alias = "_id")]
    id: Option<String>,
    name: String,
}

#[openapi]
#[post("/portfolio/position")]
pub fn portfolio_position() -> WalletResult<Json<Vec<Position>>> {
    Position::calculate_all().map(Json)
}

impl Queryable for Portfolio {
    fn collection_name() -> &'static str {
        "portfolios"
    }
}

/// # Add a portfolio
///
/// Adds a new portfolio
#[openapi]
#[post("/portfolios", data = "<portfolio>")]
pub fn add_portfolio(portfolio: Json<Portfolio>) -> WalletResult<Json<Portfolio>> {
    api_add(portfolio)
}

/// # List portfolios
///
/// Lists all portfolios
#[openapi]
#[get("/portfolios?<id>&<options..>")]
pub fn get_portfolios(
    id: Option<String>,
    options: Option<Form<ListingOptions>>,
) -> WalletResult<Rest<Json<Vec<Portfolio>>>> {
    api_get::<Portfolio>(id, options)
}

/// # Get portfolio
///
/// Get a specific portfolio
#[openapi]
#[get("/portfolios/<oid>")]
pub fn get_portfolio_by_oid(oid: String) -> WalletResult<Json<Portfolio>> {
    api_get_one::<Portfolio>(oid)
}

/// # Update a portfolio
///
/// Update a specific portfolio
#[openapi]
#[put("/portfolios/<oid>", data = "<portfolio>")]
pub fn update_portfolio_by_oid(
    oid: String,
    portfolio: Json<Portfolio>,
) -> WalletResult<Json<Portfolio>> {
    api_update::<Portfolio>(oid, portfolio)
}

/// # Delete a portfolio
///
/// Delete a specific portfolio
#[openapi]
#[delete("/portfolios/<oid>")]
pub fn delete_portfolio_by_oid(oid: String) -> WalletResult<Json<Portfolio>> {
    api_delete::<Portfolio>(oid)
}
