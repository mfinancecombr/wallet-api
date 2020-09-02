use clokwerk;
use clokwerk::TimeUnits;
use log::{info, warn};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::Rocket;
use std::collections::HashSet;
use std::sync::Mutex;

use crate::historical::Historical;
use crate::position::Position;
use crate::walletdb::WalletDB;

pub struct LockMap(HashSet<(String, String)>);
lazy_static! {
    static ref LOCK_MAP: Mutex<LockMap> = Mutex::new(LockMap::new());
}

pub struct LockGuard(String, String);

impl Drop for LockGuard {
    fn drop(&mut self) {
        LockMap::unlock(&self.0, &self.1);
    }
}

impl LockMap {
    pub fn new() -> Self {
        LockMap(HashSet::<(String, String)>::new())
    }

    pub fn lock(collection: &str, symbol: &str) -> LockGuard {
        loop {
            if let Some(guard) = Self::try_lock(collection, symbol) {
                return guard;
            } else {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }
    }

    pub fn try_lock(collection: &str, symbol: &str) -> Option<LockGuard> {
        let tuple = (collection.to_string(), symbol.to_string());
        LOCK_MAP
            .lock()
            .map(|mut lock_map| {
                if lock_map.0.contains(&tuple) {
                    None
                } else {
                    lock_map.0.insert(tuple.clone());
                    Some(LockGuard(tuple.0, tuple.1))
                }
            })
            .expect("Failed to lock static lock map")
    }

    pub fn unlock(collection: &str, symbol: &str) {
        let tuple = (collection.to_string(), symbol.to_string());
        LOCK_MAP
            .lock()
            .map(|mut lock_map| {
                lock_map.0.remove(&tuple);
            })
            .expect("Failed to lock static lock map");
    }
}

pub struct Scheduler {
    inner: Mutex<clokwerk::Scheduler>,
}

impl Fairing for Scheduler {
    fn info(&self) -> Info {
        Info {
            name: "Wallet Scheduler",
            kind: Kind::Launch,
        }
    }

    fn on_launch(&self, rocket: &Rocket) {
        let db = WalletDB::get_one(&rocket).expect("Could not get DB connection");

        std::thread::spawn(move || {
            info!("=> Starting on-launch full refresh…");

            if let Err(e) = Historical::refresh_all(&db) {
                warn!("failed to pre-calculate historicals: {:?}", e);
            }

            info!("=> Done refreshing historicals…");

            if let Err(e) = Position::calculate_all(&db) {
                warn!("failed to pre-calculate positions: {:?}", e);
            }

            info!("=> Done calculating position snapshots. On-launch refresh complete.");
        });

        self.inner
            .lock()
            .map(|mut scheduler| {
                scheduler.every(1.day()).at("2:00 am");
            })
            .expect("Failure locking Scheduler mutex");
    }
}

impl Scheduler {
    pub fn fairing() -> Self {
        Scheduler {
            inner: Mutex::new(clokwerk::Scheduler::new()),
        }
    }
}
