pub use crate::prelude::*;
use std::ops::{Index, IndexMut};

/// Holds a given color value and offers several import and export methods.
#[derive(Clone, Debug)]
pub struct TheColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl TheColor {
    /// Creates a color from u8 values.
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Creates a color from u8 values.
    pub fn from_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    /// Creates a color from u8 values.
    pub fn from_u8_array(color: [u8; 4]) -> Self {
        Self {
            r: color[0] as f32 / 255.0,
            g: color[1] as f32 / 255.0,
            b: color[2] as f32 / 255.0,
            a: color[3] as f32 / 255.0,
        }
    }

    /// Creates a white color.
    pub fn white() -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }

    /// Creates a black color.
    pub fn black() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }

    /// Creates an [f32;4] array
    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Creates an [u8;4] array
    pub fn to_u8_array(&self) -> [u8; 4] {
        [
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
            (self.a * 255.0) as u8,
        ]
    }

    pub fn as_srgba(&self) -> TheColor {
        TheColor::new(
            powf(self.r, 0.45),
            powf(self.g, 0.45),
            powf(self.b, 0.45),
            powf(self.a, 0.45),
        )
    }
}

impl Index<usize> for TheColor {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.r,
            1 => &self.g,
            2 => &self.b,
            3 => &self.a,
            _ => panic!("Index out of bounds!"),
        }
    }
}

impl IndexMut<usize> for TheColor {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.r,
            1 => &mut self.g,
            2 => &mut self.b,
            3 => &mut self.a,
            _ => panic!("Index out of bounds!"),
        }
    }
}
