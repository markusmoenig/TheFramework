use crate::prelude::*;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct TheTileMask {
    units: f32,
    pixels: Vec<Vec2f>, // Store pixels as logical coordinates.
}

impl Default for TheTileMask {
    fn default() -> Self {
        Self::new_with_12_units()
    }
}

impl TheTileMask {
    // Initialize a new TheTileMask.
    pub fn new_with_12_units() -> Self {
        TheTileMask {
            pixels: Vec::new(),
            units: 12.0,
        }
    }

    /// Returns true if the tile mask is empty.
    pub fn is_empty(&self) -> bool {
        self.pixels.is_empty()
    }

    /// Returns true if the physical pixel is contained in the tile mask.
    pub fn contains(&self, physical: Vec2i, tile_size: i32) -> bool {
        let comparison_logical_x = physical.x as f32 / tile_size as f32;
        let comparison_logical_y = physical.y as f32 / tile_size as f32;

        self.pixels.iter().any(|pixel| {
            // Check for direct overlap in the logical coordinate space.
            let delta_x = (comparison_logical_x - pixel.x).abs();
            let delta_y = (comparison_logical_y - pixel.y).abs();

            // Determine overlap considering the original and comparison tile sizes.
            delta_x < 1.0 / self.units && delta_y < 1.0 / self.units
        })
    }

    // Add a pixel.
    pub fn add_pixel(&mut self, physical: Vec2i, tile_size: i32) {
        let logical_x = physical.x as f32 / tile_size as f32;
        let logical_y = physical.y as f32 / tile_size as f32;
        self.pixels.push(Vec2f {
            x: logical_x,
            y: logical_y,
        });
    }

    // Remove a pixel pixel.
    pub fn remove_pixel(&mut self, physical: Vec2i, tile_size: i32) {
        let logical_x = physical.x as f32 / tile_size as f32;
        let logical_y = physical.y as f32 / tile_size as f32;
        self.pixels
            .retain(|pixel| pixel.x != logical_x || pixel.y != logical_y);
    }
}
