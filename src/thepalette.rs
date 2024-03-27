pub use crate::prelude::*;
use std::ops::{Index, IndexMut};

/// Holds an array of colors.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ThePalette {
    #[serde(default)]
    pub current_index: u16,
    pub colors: Vec<Option<TheColor>>,
}

impl Default for ThePalette {
    fn default() -> Self {
        Self::empty_256()
    }
}

impl ThePalette {
    pub fn new(colors: Vec<Option<TheColor>>) -> Self {
        Self {
            current_index: 0,
            colors,
        }
    }

    pub fn empty_256() -> Self {
        let mut colors = Vec::new();
        for _ in 0..256 {
            colors.push(None);
        }
        Self {
            current_index: 0,
            colors,
        }
    }

    /// Get the color at the current index
    pub fn get_current_color(&self) -> Option<TheColor> {
        self.colors[self.current_index as usize].clone()
    }
}

impl Index<usize> for ThePalette {
    type Output = Option<TheColor>;

    fn index(&self, index: usize) -> &Self::Output {
        if index < self.colors.len() {
            &self.colors[index]
        } else {
            panic!("Color Index out of bounds!");
        }
    }
}

impl IndexMut<usize> for ThePalette {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index < self.colors.len() {
            &mut self.colors[index]
        } else {
            panic!("Color Index out of bounds!");
        }
    }
}
