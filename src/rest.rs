use okapi::openapi3::Responses;
use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{Responder, Response};
use rocket_contrib::json::Json;
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::response::OpenApiResponder;
use rocket_okapi::util::add_schema_response;

use crate::error::{BackendError};
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
    where R: Responder<'r>
{
    fn responses(gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        let mut responses = Responses::default();
        let schema = gen.json_schema::<String>();
        add_schema_response(&mut responses, 200, "application/json", schema.clone())?;
        Ok(responses)
    }
}

pub fn api_add<'de, T>(db: WalletDB, operation: Json<T>) -> Result<Json<T>, BackendError>
    where T: Queryable<'de>
{
    insert_one::<T>(&*db, operation.into_inner()).map(|result| Json(result))
}

pub fn api_get<'de, T>(db: WalletDB) -> Result<Rest<Json<Vec<T>>>, BackendError>
    where T: Queryable<'de>
{
    get::<T>(&*db).map(|results| {
        let count = results.len();
        Rest(Json(results), count)
    })
}

pub fn api_get_one<'de, T>(db: WalletDB, oid: String) -> Result<Json<T>, BackendError>
    where T: Queryable<'de>
{
    get_one::<T>(&*db, oid).map(|results| Json(results))
}

pub fn api_update<'de, T>(db: WalletDB, oid: String, operation: Json<T>) -> Result<Json<T>, BackendError>
    where T: Queryable<'de>
{
    update_one::<T>(&*db, oid, operation.into_inner()).map(|result| Json(result))
}

pub fn api_delete<'de, T>(db: WalletDB, oid: String) -> Result<Json<T>, BackendError>
    where T: Queryable<'de>
{
    delete_one::<T>(&*db, oid).map(|result| Json(result))
}