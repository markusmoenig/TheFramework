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

    /// Clears all palette colors.
    pub fn clear(&mut self) {
        for v in self.colors.iter_mut() {
            *v = None;
        }
    }

    /// Load the palette from a Paint.net TXT file
    pub fn load_from_txt(&mut self, txt: String) {
        let mut index = self.current_index as usize;
        for line in txt.lines() {
            // Ignore comments
            if line.starts_with(';') {
                continue;
            }

            let mut chars = line.chars();

            // Skip Alpha
            if chars.next().is_none() {
                return;
            }
            if chars.next().is_none() {
                return;
            }

            // R
            let mut r_string = "".to_string();
            if let Some(c) = chars.next() {
                r_string.push(c);
            }
            if let Some(c) = chars.next() {
                r_string.push(c);
            }

            let r = u8::from_str_radix(&r_string, 16);

            // G
            let mut g_string = "".to_string();
            if let Some(c) = chars.next() {
                g_string.push(c);
            }
            if let Some(c) = chars.next() {
                g_string.push(c);
            }

            let g = u8::from_str_radix(&g_string, 16);

            // B
            let mut b_string = "".to_string();
            if let Some(c) = chars.next() {
                b_string.push(c);
            }
            if let Some(c) = chars.next() {
                b_string.push(c);
            }

            let b = u8::from_str_radix(&b_string, 16);

            if r.is_ok() && g.is_ok() && b.is_ok() {
                let r = r.ok().unwrap();
                let g = g.ok().unwrap();
                let b = b.ok().unwrap();

                if index < self.colors.len() {
                    self.colors[index] = Some(TheColor::from_u8(r, g, b, 0xFF));
                }

                index += 1;
            }
        }
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
