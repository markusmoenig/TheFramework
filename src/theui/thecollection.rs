pub use crate::prelude::*;
use std::collections::BTreeMap;

/// Represents a collection of TheValues.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct TheCollection {
    pub name: String,
    pub keys: BTreeMap<String, TheValue>,
}

impl Default for TheCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl TheCollection {
    pub fn named(name: String) -> Self {
        Self {
            name,
            keys: BTreeMap::default(),
        }
    }
    pub fn new() -> Self {
        Self {
            name: str!("Unnamed"),
            keys: BTreeMap::default(),
        }
    }

    /// Returns the given key.
    pub fn get(&self, key: &str) -> Option<&TheValue> {
        self.keys.get(key)
    }

    /// Returns the given key, if not found return the default.
    pub fn get_default(&self, key: &str, default: TheValue) -> TheValue {
        if let Some(v) = self.keys.get(key) {
            v.clone()
        } else {
            default
        }
    }

    /// Sets the given key with the given value.
    pub fn set(&mut self, key: &str, value: TheValue) {
        self.keys.insert(key.to_string(), value);
    }
}
