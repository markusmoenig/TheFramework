use crate::prelude::*;

pub struct TheCodeContext {

    code: FxHashMap<(u32, u32), TheAtom>
}

impl Default for TheCodeContext {
    fn default() -> Self {
        TheCodeContext::new()
    }
}

impl TheCodeContext {
    pub fn new() -> Self {
        Self {
            code: FxHashMap::default()
        }
    }
}
