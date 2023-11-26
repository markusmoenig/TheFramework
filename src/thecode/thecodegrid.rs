use crate::prelude::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TheCodeGrid {
    pub code: FxHashMap<(u32, u32), TheAtom>,
}

impl Default for TheCodeGrid {
    fn default() -> Self {
        TheCodeGrid::new()
    }
}

impl TheCodeGrid {
    pub fn new() -> Self {
        Self {
            code: FxHashMap::default(),
        }
    }
}
