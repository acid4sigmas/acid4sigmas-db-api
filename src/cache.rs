
use std::sync::{Arc, Mutex};
use lru_cache::LruCache;
use std::hash::Hash;

pub struct CacheManager<K, V>
where
    K: Eq + Hash
{
    cache: Arc<Mutex<LruCache<K, V>>>
}

impl<K,V> CacheManager<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone
{
    pub fn new(capacity: usize) -> Self {
        let cache = LruCache::new(capacity);
        let cache = Arc::new(Mutex::new(cache));
        CacheManager { cache }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        self.cache.lock().unwrap().get_mut(&key).cloned()
    }

    pub fn insert(&self, key: K, value: V) {
        self.cache.lock().unwrap().insert(key, value);
    }

    pub fn remove(&self, key: &K) -> Option<V> {
        self.cache.lock().unwrap().remove(key)
    }


}