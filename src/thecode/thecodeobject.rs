use crate::prelude::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TheCodeObject {
    pub values: FxHashMap<String, TheValue>,
}

impl Default for TheCodeObject {
    fn default() -> Self {
        TheCodeObject::new()
    }
}

impl TheCodeObject {
    pub fn new() -> Self {
        Self {
            values: FxHashMap::default(),
        }
    }

    /// Get a value in the object.
    pub fn get(&self, name: &String) -> Option<&TheValue> {
        self.values.get(name)
    }

    /// Set a value in the object.
    pub fn set(&mut self, name: String, value: TheValue) {
        self.values.insert(name, value);
    }

    /// Converts the object to a string representation with line feeds.
    pub fn describe(&self) -> String {
        let mut items: Vec<_> = self.values.iter().collect();
        items.sort_by_key(|&(key, _)| key);
        items.iter()
            .map(|(key, value)| format!("{}: {}", key, value.describe()))
            .collect::<Vec<String>>()
            .join("\n")
    }
}
