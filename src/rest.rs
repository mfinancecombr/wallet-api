use mongodb::coll::options::FindOptions;
use mongodb::{bson, doc};
use okapi::openapi3::Responses;
use rocket::http::Status;
use rocket::request::{Form, Request};
use rocket::response::{Responder, Response};
use rocket_contrib::json::Json;
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::response::OpenApiResponder;
use rocket_okapi::util::add_schema_response;
use serde::{Deserialize, Serialize};

use crate::error::WalletResult;
use crate::walletdb::*;

#[derive(Debug)]
pub struct Rest<R>(pub R, pub usize);

impl<'r, R: Responder<'r>> Responder<'r> for Rest<R> {
    fn respond_to(self, req: &Request) -> Result<Response<'r>, Status> {
        Response::build()
            .merge(self.0.respond_to(req)?)
            .raw_header("X-Total-Count", self.1.to_string())
            .ok()
    }
}

impl<'r, R> OpenApiResponder<'r> for Rest<R>
where
    R: Responder<'r>,
{
    fn responses(gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        let mut responses = Responses::default();
        let schema = gen.json_schema::<String>();
        add_schema_response(&mut responses, 200, "application/json", schema)?;
        Ok(responses)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, JsonSchema, FromForm)]
pub struct ListingOptions {
    _start: Option<i64>,
    _end: Option<i64>,
    _order: Option<String>,
    _sort: Option<String>,
}

pub fn api_add<'de, T>(db: WalletDB, operation: Json<T>) -> WalletResult<Json<T>>
where
    T: Queryable<'de>,
{
    insert_one::<T>(&*db, operation.into_inner()).map(Json)
}

pub fn api_get<'de, T>(
    db: WalletDB,
    options: Option<Form<ListingOptions>>,
) -> WalletResult<Rest<Json<Vec<T>>>>
where
    T: Queryable<'de>,
{
    let mut find_options = FindOptions::new();
    if let Some(options) = options {
        let mut limit: Option<i64> = None;
        if options._end.is_some() && options._start.is_some() {
            limit = Some(options._end.unwrap() - options._start.unwrap());
        }

        find_options.skip = options._start;
        find_options.limit = limit;
        if let Some(sort) = &options._sort {
            find_options.sort = Some(doc! {
                sort: options._order.as_ref().map(|order| {
                    if order == "DESC" {
                        1
                    } else {
                        -1
                    }
                }).unwrap_or(1)
            });
        }
    };

    let count = get_count::<T>(&*db)?;
    get::<T>(&*db, Some(find_options)).map(|results| Rest(Json(results), count as usize))
}

pub fn api_get_one<'de, T>(db: WalletDB, oid: String) -> WalletResult<Json<T>>
where
    T: Queryable<'de>,
{
    get_one::<T>(&*db, oid).map(Json)
}

pub fn api_update<'de, T>(db: WalletDB, oid: String, operation: Json<T>) -> WalletResult<Json<T>>
where
    T: Queryable<'de>,
{
    update_one::<T>(&*db, oid, operation.into_inner()).map(Json)
}

pub fn api_delete<'de, T>(db: WalletDB, oid: String) -> WalletResult<Json<T>>
where
    T: Queryable<'de>,
{
    delete_one::<T>(&*db, oid).map(Json)
}
