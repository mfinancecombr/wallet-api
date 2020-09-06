use mongodb::bson::{doc, oid, to_bson, Bson};
use mongodb::options::FindOptions;
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

pub fn api_add<T>(operation: Json<T>) -> WalletResult<Json<T>>
where
    T: Queryable,
{
    insert_one::<T>(operation.into_inner()).map(Json)
}

pub fn api_get<T>(
    id: Option<String>,
    options: Option<Form<ListingOptions>>,
) -> WalletResult<Rest<Json<Vec<T>>>>
where
    T: Queryable,
{
    let filter = id.map(|id| {
        // This is just string to Bson. It shouldn't really fail unless something went
        // quite wrong, so we just panic if it fails to convert.
        let ids_to_lookup = id
            .split(',')
            .map(|s| Bson::ObjectId(oid::ObjectId::with_string(s).unwrap()))
            .collect::<Vec<Bson>>();

        doc! {
            "_id": { "$in": to_bson(&Bson::Array(ids_to_lookup)).unwrap() }
        }
    });

    let mut find_options: Option<FindOptions> = None;
    if let Some(options) = options {
        let skip = options._start;
        let limit = {
            if options._end.is_some() && options._start.is_some() {
                Some(options._end.unwrap() - options._start.unwrap())
            } else {
                None
            }
        };
        let sort = {
            if let Some(sort) = &options._sort {
                let order = {
                    if let Some(order) = &options._order {
                        if order == "DESC" {
                            1
                        } else {
                            -1
                        }
                    } else {
                        -1
                    }
                };

                Some(doc! {
                    sort: order
                })
            } else {
                None
            }
        };

        find_options = Some(
            FindOptions::builder()
                .skip(skip)
                .limit(limit)
                .sort(sort)
                .build(),
        );
    };

    let count = get_count::<T>()?;
    get::<T>(filter, find_options).map(|results| Rest(Json(results), count as usize))
}

pub fn api_get_one<T>(oid: String) -> WalletResult<Json<T>>
where
    T: Queryable,
{
    get_one::<T>(oid).map(Json)
}

pub fn api_update<T>(oid: String, operation: Json<T>) -> WalletResult<Json<T>>
where
    T: Queryable,
{
    update_one::<T>(oid, operation.into_inner()).map(Json)
}

pub fn api_delete<T>(oid: String) -> WalletResult<Json<T>>
where
    T: Queryable,
{
    delete_one::<T>(oid).map(Json)
}
