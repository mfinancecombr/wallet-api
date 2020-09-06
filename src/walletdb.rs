use mongodb::bson::{doc, from_bson, oid, spec, to_bson, Bson, Document};
use mongodb::options::FindOptions;
use mongodb::sync::{Client, Cursor, Database};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::Rocket;
use rocket_contrib::databases::database_config;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::cell::RefCell;
use std::sync::Mutex;

use crate::error::{BackendError, WalletResult};

lazy_static! {
    static ref WALLET_CLIENT: Mutex<RefCell<Option<Client>>> = Mutex::new(RefCell::new(None));
}

#[derive(Debug)]
pub struct WalletDB {}

impl WalletDB {
    pub fn fairing() -> Self {
        WalletDB {}
    }

    pub fn init_client(uri: &str) {
        WALLET_CLIENT.lock().unwrap().replace(Some(
            Client::with_uri_str(uri).expect("Failed to connect to mongodb"),
        ));
    }

    pub fn get_connection() -> Database {
        if cfg!(test) {
            WALLET_CLIENT
                .lock()
                .unwrap()
                .borrow()
                .as_ref()
                .unwrap()
                .database("wallet-fake-test")
        } else {
            WALLET_CLIENT
                .lock()
                .unwrap()
                .borrow()
                .as_ref()
                .unwrap()
                .database("wallet")
        }
    }
}

impl Fairing for WalletDB {
    fn info(&self) -> Info {
        Info {
            name: "WalletDB",
            kind: Kind::Launch,
        }
    }

    fn on_launch(&self, rocket: &Rocket) {
        let database = database_config("wallet", rocket.config())
            .expect("Did not find database configuration in Rocket.toml");
        Self::init_client(database.url);
    }
}

pub trait Queryable: Serialize + DeserializeOwned + std::fmt::Debug {
    fn collection_name() -> &'static str;

    fn from_docs(cursor: Cursor) -> WalletResult<Vec<Self>> {
        cursor
            .map(|result| match result {
                Ok(doc) => Self::from_doc(doc),
                Err(e) => Err(dang!(Database, e)),
            })
            .collect::<WalletResult<Vec<Self>>>()
    }

    fn from_doc(doc: Document) -> WalletResult<Self> {
        // Since I don't want to rename the id field to keep the REST API pretty,
        // I need to do some surgery here. Note that some models, like Broker, use
        // slugs as their IDs instead of ObjectIds, so we need to handle the two
        // possibilities here.
        let mut doc = doc;
        if let Some(id) = doc.remove("_id") {
            match id.as_object_id() {
                Some(id) => doc.insert(String::from("id"), id.to_string()),
                None => doc.insert(String::from("id"), id),
            };
        }

        match from_bson(Bson::Document(doc)) {
            Ok(obj) => Ok(obj),
            Err(e) => Err(dang!(Bson, e)),
        }
    }

    fn to_doc(&self) -> WalletResult<Document> {
        // Reverse surgery (see above) on the id, to rename it properly. Do we need to handle
        // models with ObjectId at all?
        fn fix_id(doc: &mut Document) {
            if let Some(id) = doc.remove("id") {
                if id.element_type() != spec::ElementType::Null {
                    doc.insert(String::from("_id"), id);
                }
            }
        }

        match to_bson(self) {
            Ok(doc) => match doc {
                Bson::Document(mut doc) => {
                    fix_id(&mut doc);
                    Ok(doc)
                }
                _ => Err(dang!(Bson, "Failed to create Document")),
            },
            Err(e) => Err(dang!(Bson, e)),
        }
    }
}

pub fn get<T>(filter: Option<Document>, options: Option<FindOptions>) -> WalletResult<Vec<T>>
where
    T: Queryable,
{
    let wallet = WalletDB::get_connection();
    let cursor = match wallet
        .collection(T::collection_name())
        .find(filter, options)
    {
        Ok(cursor) => cursor,
        Err(e) => return Err(dang!(Database, e)),
    };
    T::from_docs(cursor)
}

pub fn get_count<T>() -> WalletResult<i64>
where
    T: Queryable,
{
    let wallet = WalletDB::get_connection();
    wallet
        .collection(T::collection_name())
        .count_documents(None, None)
        .map_err(|e| dang!(Database, e))
}

fn string_to_objectid(oid: &str) -> Result<oid::ObjectId, oid::Error> {
    oid::ObjectId::with_string(oid)
}

fn objectid_to_string(oid: Bson) -> WalletResult<String> {
    oid.as_object_id()
        .map(|oid| oid.to_string())
        .ok_or_else(|| dang!(Bson, format!("Could not convert {:?} to String", oid)))
}

fn filter_from_oid(oid: &str) -> Document {
    if let Ok(object_id) = string_to_objectid(oid) {
        doc! {"_id": object_id}
    } else {
        doc! {"_id": oid}
    }
}

pub fn get_one<T>(oid: String) -> WalletResult<T>
where
    T: Queryable,
{
    let wallet = WalletDB::get_connection();
    match wallet
        .collection(T::collection_name())
        .find_one(Some(filter_from_oid(&oid)), None)
    {
        Ok(doc) => doc.map_or(Err(BackendError::NotFound), T::from_doc),
        Err(e) => Err(dang!(Database, e)),
    }
}

pub fn insert_one<T>(obj: T) -> WalletResult<T>
where
    T: Queryable,
{
    let mut doc = T::to_doc(&obj)?;

    // We don't want users to specify their own ids, we want mongodb to generate them,
    // so ignore if any comes along with the request.
    doc.remove("_id");

    let wallet = WalletDB::get_connection();
    match wallet
        .collection(T::collection_name())
        .insert_one(doc, None)
    {
        Ok(result) => get_one(objectid_to_string(result.inserted_id)?),
        Err(e) => Err(dang!(Database, e)),
    }
}

pub fn update_one<T>(oid: String, obj: T) -> WalletResult<T>
where
    T: Queryable,
{
    let mut doc = T::to_doc(&obj)?;

    // $set doesn't seem to like getting data with _id, so we remove it.
    doc.remove("_id");

    let wallet = WalletDB::get_connection();
    match wallet.collection(T::collection_name()).update_one(
        filter_from_oid(&oid),
        doc! {"$set": doc},
        None,
    ) {
        Ok(_) => get_one(oid),
        Err(e) => Err(dang!(Database, e)),
    }
}

pub fn delete_one<T>(oid: String) -> WalletResult<T>
where
    T: Queryable,
{
    let result = get_one::<T>(oid.clone())?;
    let wallet = WalletDB::get_connection();
    match wallet
        .collection(T::collection_name())
        .delete_one(filter_from_oid(&oid), None)
    {
        Ok(_) => Ok(result),
        Err(e) => Err(dang!(Database, e)),
    }
}
