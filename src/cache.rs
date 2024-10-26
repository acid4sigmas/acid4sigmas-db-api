use lru_cache::LruCache;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

lazy_static::lazy_static! {
    pub static ref CACHE_MANAGER: CacheManager<String, Vec<serde_json::Value>> = CacheManager::new(1000);
}

pub struct CacheManager<K, V>
where
    K: Eq + Hash,
{
    cache: Arc<Mutex<LruCache<K, V>>>,
    keys: Arc<Mutex<Vec<K>>>,
}

impl<K, V> CacheManager<K, V>
where
    K: Eq + Hash + Clone + ToString,
    V: Clone,
{
    pub fn new(capacity: usize) -> Self {
        let cache = LruCache::new(capacity);
        let cache = Arc::new(Mutex::new(cache));
        let keys = Arc::new(Mutex::new(Vec::new()));
        CacheManager { cache, keys }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        self.cache.lock().unwrap().get_mut(&key).cloned()
    }

    pub fn insert(&self, key: K, value: V) {
        self.cache.lock().unwrap().insert(key.clone(), value);

        self.keys.lock().unwrap().push(key);
    }

    pub fn remove_by_prefix(&self, prefix: &str) {
        let mut cache = self.cache.lock().unwrap();
        let mut keys = self.keys.lock().unwrap();

        let keys_to_remove: Vec<K> = keys
            .iter()
            .filter(|key| {
                let key_str = key.to_string();
                key_str.starts_with(prefix)
            })
            .cloned()
            .collect();

        for key in keys_to_remove {
            cache.remove(&key);
            keys.retain(|k| k != &key);
        }
    }
}

use std::collections::hash_map::DefaultHasher;

pub struct CacheKey;

impl CacheKey {
    pub fn generate_cache_key(
        table_name: &str,
        query: &str,
        params: &[serde_json::Value],
    ) -> String {
        let table_hash = Self::generate_table_cache_hash(table_name);
        let query_hash = Self::generate_query_cache_hash(query);
        let params_hash = Self::generate_params_cache_hash(params);

        format!("{}_{}_{}", table_hash, query_hash, params_hash)
        // combine the hashes
    }

    pub fn generate_table_cache_hash(table_name: &str) -> String {
        let mut table_hasher = DefaultHasher::new();
        table_name.hash(&mut table_hasher);
        let table_hash = table_hasher.finish().to_string();
        table_hash
    }

    fn generate_query_cache_hash(query: &str) -> String {
        let mut query_hasher = DefaultHasher::new();
        query.hash(&mut query_hasher);
        let query_hash = query_hasher.finish().to_string();
        query_hash
    }

    fn generate_params_cache_hash(params: &[serde_json::Value]) -> String {
        let mut params_hasher = DefaultHasher::new();

        for param in params {
            param.hash(&mut params_hasher);
        }
        let params_hash = params_hasher.finish().to_string();
        params_hash
    }
}
