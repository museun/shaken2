use super::kv::KeyValueStore;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Tracker {
    pub users: Map,
    pub rooms: Map,
}

impl Default for Tracker {
    fn default() -> Self {
        Self::new()
    }
}

impl Tracker {
    pub fn new() -> Self {
        Self {
            users: Map::new("user_id_mappings"),
            rooms: Map::new("room_id_mappings"),
        }
    }
}

#[derive(Clone)]
pub struct Map {
    kv: Arc<Mutex<KeyValueStore<'static>>>,
}

impl Map {
    pub fn new(table_name: &'static str) -> Self {
        KeyValueStore::fetch(table_name)
            .map(Mutex::new)
            .map(Arc::new)
            .map(|kv| Self { kv })
            .unwrap()
    }

    pub async fn extend<I>(&self, iter: I)
    where
        I: IntoIterator<Item = (u64, String)>,
        I::IntoIter: Send + 'static,
        I::Item: Send,
    {
        let iter = iter.into_iter();
        let kv = Arc::clone(&self.kv);
        tokio::task::spawn_blocking(move || {
            let kv = kv.lock().unwrap();
            for (key, val) in iter {
                kv.set(&key, &val).unwrap();
            }
        })
        .await
        .unwrap();
    }

    pub async fn get(&self, key: u64) -> Option<String> {
        let kv = Arc::clone(&self.kv);
        tokio::task::spawn_blocking(move || {
            log::trace!("getting id: {}", key);
            kv.lock().unwrap().get(&key)
        })
        .await
        .unwrap()
    }

    pub async fn set(&self, key: u64, val: impl ToString) {
        let val = val.to_string();
        let kv = Arc::clone(&self.kv);
        tokio::task::spawn_blocking(move || {
            log::trace!("setting id {} to '{}'", key, val.escape_debug());
            let kv = kv.lock().unwrap();
            kv.set(&key, &val).unwrap();
        })
        .await
        .unwrap();
    }
}
