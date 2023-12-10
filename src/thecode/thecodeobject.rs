use crate::prelude::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
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
}
