use crate::prelude::*;

/// TheCodeLevel holds all necessary data needed to represent a game level from a CodeGridFX.
/// i.e. defining blocking areas, spawn points, portals, tile types at a given position etc.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TheCodeLevel {
    blocking: FxHashSet<(u16, u16)>,
}

impl Default for TheCodeLevel {
    fn default() -> Self {
        TheCodeLevel::new()
    }
}

impl TheCodeLevel {
    pub fn new() -> Self {
        Self {
            blocking: FxHashSet::default(),
        }
    }

    /// Clears the blocking positions of the level.
    pub fn clear_blocking(&mut self) {
        self.blocking.clear();
    }

    /// Marks the given position as blocking.
    #[inline(always)]
    pub fn set_blocking(&mut self, position: (u16, u16)) {
        self.blocking.insert(position);
    }

    /// Checks if the given position is blocking.
    #[inline(always)]
    pub fn is_blocking(&mut self, position: (u16, u16)) -> bool {
        self.blocking.contains(&position)
    }
}
