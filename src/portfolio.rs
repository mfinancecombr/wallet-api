use chrono::{Date, Utc};
use rocket::request::Form;
use rocket_contrib::json::Json;
use rocket_okapi::{openapi, JsonSchema};
use serde::{Deserialize, Serialize};

use crate::error::WalletResult;
use crate::operation::OperationKind;
use crate::position::Position;
use crate::rest::*;
use crate::walletdb::Queryable;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
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
    let mut result = Position::get_all_for_portfolio(id)?;
    let count = result.len();

    if let Some(options) = options {
        if let Some(sort) = options._sort.as_ref() {
            match sort.as_str() {
                "id" => result.sort_by(Position::cmp_id),
                "symbol" => result.sort_by(Position::cmp_symbol),
                "quantity" => result.sort_by(Position::cmp_quantity),
                "average_price" => result.sort_by(Position::cmp_average_price),
                "current_price" => result.sort_by(Position::cmp_current_price),
                "cost_basis" => result.sort_by(Position::cmp_cost_basis),
                "current_value" => result.sort_by(Position::cmp_current_value),
                "gain" => result.sort_by(Position::cmp_gain),
                _ => unimplemented!(),
            }
        }

        if let Some(order) = options._order.as_ref() {
            if let "DESC" = order.as_str() {
                result.reverse();
            }
        }

        let start = std::cmp::min(options._start.unwrap_or(0) as usize, count as usize);
        let end = std::cmp::min(options._end.unwrap_or(10) as usize, count as usize);

        let result = (&result[start..end]).to_vec();

        Ok(Rest(Json(result), count))
    } else {
        Ok(Rest(Json(result), count))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct PerformanceSnapshot {
    name: String,
    reference: f64,
    percentual_gain: f64,
}

/// # Obtain cash-flow-adjusted historical performance
///
/// Returns an array with weekly snapshots of portfolio performance adjusted
/// by purchases and sales on that week. This gives us performance measurements
/// that can be compared with indexes or funds performance.
#[openapi]
#[get("/portfolios/performance?<oid>")]
pub fn performance(oid: Option<String>) -> WalletResult<Json<Vec<PerformanceSnapshot>>> {
    let snapshots = Position::get_history_for_portfolio(oid, None)?;
    let mut dates = snapshots.keys().collect::<Vec<&Date<Utc>>>();
    dates.sort();

    #[derive(Copy, Clone, Debug)]
    struct AggregatePosition {
        cost_basis: f64,
        current_value: f64,
        operations_adjustment: f64,
    }

    let mut performance_snapshots = vec![];
    let mut previous_aggregate: Option<AggregatePosition> = None;
    let mut snapshot = PerformanceSnapshot {
        name: String::new(),
        reference: 100.,
        percentual_gain: 0.,
    };

    for d in dates {
        let positions = snapshots.get(d).unwrap();
        let aggregate = positions.iter().fold(
            AggregatePosition {
                cost_basis: 0.,
                current_value: 0.,
                operations_adjustment: 0.,
            },
            |acc, pos| {
                let mut acc = acc;
                acc.cost_basis += pos.cost_basis;
                acc.current_value += pos.current_price * pos.quantity as f64;

                // We need to adjust our aggregate numbers for the operations
                // that happened since our last snapshot, so that they do
                // not affect our return calculation.
                for op in &pos.recent_operations {
                    let adjustment = op.quantity as f64 * op.price;
                    match op.kind {
                        OperationKind::Purchase => acc.operations_adjustment += adjustment,
                        OperationKind::Sale => acc.operations_adjustment -= adjustment,
                    }
                }
                acc
            },
        );

        let mut percent_change = 0.;
        if let Some(previous_aggregate) = previous_aggregate {
            let adjusted_current_value = aggregate.current_value - aggregate.operations_adjustment;
            percent_change = (adjusted_current_value - previous_aggregate.current_value)
                / previous_aggregate.current_value;
        }

        snapshot.name = d.naive_utc().to_string();
        snapshot.reference += snapshot.reference.abs() * percent_change;
        snapshot.percentual_gain = snapshot.reference - 100.;

        performance_snapshots.push(snapshot.clone());

        previous_aggregate = Some(aggregate);
    }

    Ok(Json(performance_snapshots))
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
