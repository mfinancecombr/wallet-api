use std::collections::HashSet;
use std::sync::Mutex;


pub struct LockMap(HashSet<(String, String)>);
lazy_static! {
    static ref LOCK_MAP: Mutex<LockMap> = Mutex::new(LockMap::new());
}

pub struct LockGuard((String, String));

impl Drop for LockGuard {
    fn drop(&mut self) {
        LockMap::unlock(&self.0.0, &self.0.1);
    }
}

impl LockMap {
    pub fn new() -> Self {
        LockMap(HashSet::<(String, String)>::new())
    }

    pub fn lock(collection: &str, symbol: &str) -> LockGuard {
        loop {
            if let Some(guard) = Self::try_lock(collection, symbol) {
                return guard
            } else {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        };
    }

    pub fn try_lock(collection: &str, symbol: &str) -> Option<LockGuard> {
        let tuple = (collection.to_string(), symbol.to_string());
        LOCK_MAP.lock().map(|mut lock_map| {
            if lock_map.0.contains(&tuple) {
                None
            } else {
                lock_map.0.insert(tuple.clone());
                Some(LockGuard(tuple))
            }
        }).expect("Failed to lock static lock map")
    }

    pub fn unlock(collection: &str, symbol: &str) {
        let tuple = (collection.to_string(), symbol.to_string());
        LOCK_MAP.lock().map(|mut lock_map| {
            lock_map.0.remove(&tuple);
        }).expect("Failed to lock static lock map");
    }
}
