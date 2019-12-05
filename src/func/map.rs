//! A deduplicating map.

use std::collections::HashMap;
use std::hash::Hash;

use crate::syntax::{Spanned, ParseResult};

/// A deduplicating map type useful for storing possibly redundant arguments.
#[derive(Debug, Clone, PartialEq)]
pub struct ConsistentMap<K, V> where K: Hash + Eq {
    map: HashMap<K, V>,
}

impl<K, V> ConsistentMap<K, V> where K: Hash + Eq {
    pub fn new() -> ConsistentMap<K, V> {
        ConsistentMap { map: HashMap::new() }
    }

    /// Add a key-value pair.
    pub fn add(&mut self, key: K, value: V) -> ParseResult<()> {
        self.map.insert(key, value);
        // TODO
        Ok(())
    }

    /// Add a key-value pair if the value is not `None`.
    pub fn add_opt(&mut self, key: K, value: Option<V>) -> ParseResult<()> {
        Ok(if let Some(value) = value {
            self.add(key, value)?;
        })
    }

    /// Add a key-spanned-value pair the value is not `None`.
    pub fn add_opt_span(&mut self, key: K, value: Option<Spanned<V>>) -> ParseResult<()> {
        Ok(if let Some(spanned) = value {
            self.add(key, spanned.v)?;
        })
    }

    /// Call a function with the value if the key is present.
    pub fn with<F>(&self, key: K, callback: F) where F: FnOnce(&V) {
        if let Some(value) = self.map.get(&key) {
            callback(value);
        }
    }

    /// Create a new consistent map where keys and values are mapped to new
    /// keys and values. Returns an error if a new key is duplicate.
    pub fn dedup<F, K2, V2>(&self, _f: F) -> ParseResult<ConsistentMap<K2, V2>>
    where F: FnOnce(K, V) -> ParseResult<(K2, V2)>, K2: Hash + Eq {
        // TODO
        Ok(ConsistentMap::new())
    }

    /// Iterate over the (key, value) pairs.
    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, K, V> {
        self.map.iter()
    }
}
