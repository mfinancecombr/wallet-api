use mongodb::coll::options::FindOptions;
use mongodb::db::ThreadedDatabase;
use mongodb::ThreadedClient;
use mongodb::{bson, doc, Bson};
use rocket_contrib::database;
use rocket_contrib::databases::mongodb;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::error::{BackendError, WalletResult};

#[database("wallet")]
pub struct WalletDB(mongodb::db::Database);

// Had to implement this manually as derive was not being picked up?
impl fmt::Debug for WalletDB {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl WalletDB {
    pub fn get_connection() -> mongodb::db::Database {
        // FIXME: use the same configuration as we have for the pool.
        let db_client = mongodb::Client::with_uri("mongodb://127.0.0.1:27017/")
            .expect("Could not connect to mongodb");

        if cfg!(test) {
            db_client.db("wallet-fake-test")
        } else {
            db_client.db("wallet")
        }
    }
}

pub trait Queryable<'de>: Serialize + Deserialize<'de> + std::fmt::Debug {
    fn collection_name() -> &'static str;

    fn from_docs(cursor: mongodb::cursor::Cursor) -> WalletResult<Vec<Self>> {
        cursor
            .map(|result| match result {
                Ok(doc) => Self::from_doc(doc),
                Err(e) => Err(dang!(Database, e)),
            })
            .collect::<WalletResult<Vec<Self>>>()
    }

    fn from_doc(doc: mongodb::ordered::OrderedDocument) -> WalletResult<Self> {
        // Since I don't want to rename the id field to keep the REST API pretty,
        // I need to do some surgery here. Note that some models, like Broker, use
        // slugs as their IDs instead of ObjectIds, so we need to handle the two
        // possibilities here.
        let mut doc = doc;
        if let Some(id) = doc.remove("_id") {
            match id.as_object_id() {
                Some(id) => doc.insert(String::from("id"), id.to_string()),
                None => doc.insert_bson(String::from("id"), id),
            };
        }

        match bson::from_bson(Bson::Document(doc)) {
            Ok(obj) => Ok(obj),
            Err(e) => Err(dang!(Bson, e)),
        }
    }

    fn to_doc(&self) -> WalletResult<mongodb::ordered::OrderedDocument> {
        // Reverse surgery (see above) on the id, to rename it properly. Do we need to handle
        // models with ObjectId at all?
        fn fix_id(doc: &mut mongodb::ordered::OrderedDocument) {
            if let Some(id) = doc.remove("id") {
                if id.element_type() != bson::spec::ElementType::NullValue {
                    doc.insert_bson(String::from("_id"), id);
                }
            }
        }

        match bson::to_bson(self) {
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

pub fn get<'de, T>(
    wallet: &mongodb::db::Database,
    options: Option<FindOptions>,
) -> WalletResult<Vec<T>>
where
    T: Queryable<'de>,
{
    let cursor = match wallet.collection(T::collection_name()).find(None, options) {
        Ok(cursor) => cursor,
        Err(e) => return Err(dang!(Database, e)),
    };
    T::from_docs(cursor)
}

pub fn get_count<'de, T>(wallet: &mongodb::db::Database) -> WalletResult<i64>
where
    T: Queryable<'de>,
{
    wallet
        .collection(T::collection_name())
        .count(None, None)
        .map_err(|e| dang!(Database, e))
}

fn string_to_objectid(oid: &str) -> Result<bson::oid::ObjectId, bson::oid::Error> {
    bson::oid::ObjectId::with_string(oid)
}

fn objectid_to_string(oid: Option<bson::Bson>) -> WalletResult<String> {
    let oid = oid.ok_or_else(|| {
        dang!(
            Bson,
            "Tried to use None as ObjectId when converting to String"
        )
    })?;
    oid.as_object_id()
        .map(|oid| oid.to_string())
        .ok_or_else(|| dang!(Bson, format!("Could not convert {:?} to String", oid)))
}

fn filter_from_oid(oid: &str) -> bson::ordered::OrderedDocument {
    if let Ok(object_id) = string_to_objectid(oid) {
        doc! {"_id": object_id}
    } else {
        doc! {"_id": oid}
    }
}

pub fn get_one<'de, T>(wallet: &mongodb::db::Database, oid: String) -> WalletResult<T>
where
    T: Queryable<'de>,
{
    match wallet
        .collection(T::collection_name())
        .find_one(Some(filter_from_oid(&oid)), None)
    {
        Ok(doc) => doc.map_or(Err(BackendError::NotFound), T::from_doc),
        Err(e) => Err(dang!(Database, e)),
    }
}

pub fn insert_one<'de, T>(wallet: &mongodb::db::Database, obj: T) -> WalletResult<T>
where
    T: Queryable<'de>,
{
    let mut doc = T::to_doc(&obj)?;

    // We don't want users to specify their own ids, we want mongodb to generate them,
    // so ignore if any comes along with the request.
    doc.remove("_id");

    match wallet
        .collection(T::collection_name())
        .insert_one(doc, None)
    {
        Ok(result) => get_one(wallet, objectid_to_string(result.inserted_id)?),
        Err(e) => Err(dang!(Database, e)),
    }
}

pub fn update_one<'de, T>(wallet: &mongodb::db::Database, oid: String, obj: T) -> WalletResult<T>
where
    T: Queryable<'de>,
{
    let mut doc = T::to_doc(&obj)?;

    // $set doesn't seem to like getting data with _id, so we remove it.
    doc.remove("_id");

    match wallet.collection(T::collection_name()).update_one(
        filter_from_oid(&oid),
        doc! {"$set": doc},
        None,
    ) {
        Ok(_) => get_one(wallet, oid),
        Err(e) => Err(dang!(Database, e)),
    }
}

pub fn delete_one<'de, T>(wallet: &mongodb::db::Database, oid: String) -> WalletResult<T>
where
    T: Queryable<'de>,
{
    let result = get_one::<T>(wallet, oid.clone())?;
    match wallet
        .collection(T::collection_name())
        .delete_one(filter_from_oid(&oid), None)
    {
        Ok(_) => Ok(result),
        Err(e) => Err(dang!(Database, e)),
    }
}
