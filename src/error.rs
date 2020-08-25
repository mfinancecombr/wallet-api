use okapi::openapi3::Responses;
use rocket::{Request, Response};
use rocket::http::Status;
use rocket::response::Responder;
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::response::OpenApiResponder;
use rocket_okapi::util::add_schema_response;
use std::io::Cursor;


#[derive(Clone, Debug, JsonSchema)]
pub enum BackendError {
  Bson(String),
  Database(String),
  NotFound,
  Yahoo(String)
}

impl Responder<'static> for BackendError {
    fn respond_to(self, _: &Request) -> Result<Response<'static>, Status> {
        let body;
        let status = match self {
            BackendError::Bson(msg) => {
            body = msg.clone();
            Status::new(500, "Bson")
          },
          BackendError::Database(msg) => {
            body = msg.clone();
            Status::new(500, "Database")
          },
          BackendError::NotFound => {
            body = String::new();
            Status::NotFound
          },
          BackendError::Yahoo(msg) => {
            body = msg.clone();
            Status::new(500, "Yahoo")
          }
        };
        Response::build()
          .status(status)
          .sized_body(Cursor::new(body))
          .ok()
        }
}

impl OpenApiResponder<'static> for BackendError {
    fn responses(gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        let mut responses = Responses::default();
        let schema = gen.json_schema::<String>();
        add_schema_response(&mut responses, 500, "text/plain", schema.clone())?;
        add_schema_response(&mut responses, 404, "text/plain", schema.clone())?;
        Ok(responses)
    }
}
