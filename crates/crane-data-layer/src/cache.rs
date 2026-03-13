// Cache Implementation

use std::collections::HashMap;

/// Cache trait - cache abstraction
pub trait Cache<K, V> {
    fn get(&self, key: &K) -> Option<&V>;
    fn set(&mut self, key: K, value: V);
    fn remove(&mut self, key: &K) -> Option<V>;
    fn clear(&mut self);
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// LRUCache - LRU cache implementation
pub struct LRUCache<K, V>
where
    K: Eq + std::hash::Hash + Clone,
{
    capacity: usize,
    cache: HashMap<K, V>,
}

impl<K, V> LRUCache<K, V>
where
    K: Eq + std::hash::Hash + Clone,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            cache: HashMap::new(),
        }
    }
}

impl<K, V> Cache<K, V> for LRUCache<K, V>
where
    K: Eq + std::hash::Hash + Clone,
{
    fn get(&self, key: &K) -> Option<&V> {
        self.cache.get(key)
    }

    fn set(&mut self, key: K, value: V) {
        if self.cache.len() >= self.capacity && !self.cache.contains_key(&key) {
            // Simple eviction - remove first entry
            if let Some(first_key) = self.cache.keys().next().cloned() {
                self.cache.remove(&first_key);
            }
        }
        self.cache.insert(key, value);
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        self.cache.remove(key)
    }

    fn clear(&mut self) {
        self.cache.clear();
    }

    fn len(&self) -> usize {
        self.cache.len()
    }
}
