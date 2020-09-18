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
    pub id: Option<String>,
    pub name: String,
}

/// # List positions
///
/// Lists all positions
#[openapi]
#[get("/positions?<options..>")]
pub fn positions(options: Option<Form<ListingOptions>>) -> WalletResult<Rest<Json<Vec<Position>>>> {
    get_portfolio_positions(None, options)
}

#[openapi]
#[get("/portfolios/positions?<id>&<options..>")]
pub fn portfolio_positions(
    id: String,
    options: Option<Form<ListingOptions>>,
) -> WalletResult<Rest<Json<Vec<Position>>>> {
    get_portfolio_positions(Some(id), options)
}

fn get_portfolio_positions(
    id: Option<String>,
    options: Option<Form<ListingOptions>>,
) -> WalletResult<Rest<Json<Vec<Position>>>> {
    let measure = std::time::Instant::now();
    let result = Position::get_all_for_portfolio(id)?;
    let count = result.len();
    println!("{}:{} {}", file!(), line!(), measure.elapsed().as_millis());

    if let Some(options) = options {
        let start = std::cmp::min(options._start.unwrap_or(0) as usize, count as usize);
        let end = std::cmp::min(options._end.unwrap_or(10) as usize, count as usize);
        Ok(Rest(Json((&result[start..end]).to_vec()), count))
    } else {
        Ok(Rest(Json(result), count))
    }
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
