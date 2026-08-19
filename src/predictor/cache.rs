use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct CacheEntry<T> {
    pub value: T,
    pub created_at: Instant,
}

pub struct TimedCache<K, V> {
    entries: HashMap<K, CacheEntry<V>>,
    ttl: Duration,
}

impl<K: std::hash::Hash + Eq + Clone, V: Clone> TimedCache<K, V> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            entries: HashMap::new(),
            ttl,
        }
    }

    pub fn get(&mut self, key: &K) -> Option<V> {
        if let Some(entry) = self.entries.get(key) {
            if entry.created_at.elapsed() <= self.ttl {
                return Some(entry.value.clone());
            }
        }
        self.entries.remove(key);
        None
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.entries.insert(
            key,
            CacheEntry {
                value,
                created_at: Instant::now(),
            },
        );
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}
