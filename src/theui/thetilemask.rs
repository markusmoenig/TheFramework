use crate::prelude::*;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct TheTileMask {
    #[serde(with = "vectorize")]
    pub pixels: FxHashMap<Vec2i, bool>,
}

impl Default for TheTileMask {
    fn default() -> Self {
        Self::new()
    }
}

impl TheTileMask {
    // Initialize a new TheTileMask.
    pub fn new() -> Self {
        TheTileMask {
            pixels: FxHashMap::default(),
        }
    }

    /// Returns true if the tile mask is empty.
    pub fn is_empty(&self) -> bool {
        self.pixels.is_empty()
    }

    /// Returns true if the physical pixel is contained in the tile mask.
    pub fn contains(&self, position: Vec2i) -> bool {
        self.pixels.contains_key(&position)
    }

    // Add a pixel.
    pub fn add_pixel(&mut self, position: Vec2i, value: bool) {
        self.pixels.insert(position, value);
    }

    // Remove a pixel pixel.
    pub fn remove_pixel(&mut self, position: Vec2i) {
        self.pixels.remove(&position);
    }
}
