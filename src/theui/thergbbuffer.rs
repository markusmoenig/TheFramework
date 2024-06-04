use super::{compress, decompress};
use crate::prelude::*;
use arboard::{Clipboard, ImageData};
use png::{BitDepth, ColorType, Encoder};

#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Clone, Debug)]
pub struct TheRGBBuffer {
    dim: TheDim,

    #[serde(serialize_with = "compress", deserialize_with = "decompress")]
    buffer: Vec<u8>,
}

impl Default for TheRGBBuffer {
    fn default() -> Self {
        Self::empty()
    }
}

/// TheRGBABuffer contains the pixel buffer for a canvas or icon.
impl TheRGBBuffer {
    /// Create an empty buffer.
    pub fn empty() -> Self {
        Self {
            dim: TheDim::zero(),
            buffer: vec![],
        }
    }

    /// Creates a buffer of the given dimension.
    pub fn new(dim: TheDim) -> Self {
        Self {
            dim,
            buffer: vec![0; dim.width as usize * dim.height as usize * 3],
        }
    }

    /// Creates a buffer from existing data.
    pub fn from(buffer: Vec<u8>, width: u32, height: u32) -> Self {
        Self {
            dim: TheDim::new(0, 0, width as i32, height as i32),
            buffer,
        }
    }

    /// Resizes the buffer.
    pub fn resize(&mut self, width: i32, height: i32) {
        if self.dim.width != width || self.dim.height != height {
            self.dim.width = width;
            self.dim.height = height;
            self.allocate();
        }
    }

    /// Check for size validity
    pub fn is_valid(&self) -> bool {
        self.dim.is_valid()
    }

    /// Gets the width (stride) of the buffer.
    pub fn dim(&self) -> &TheDim {
        &self.dim
    }

    /// Gets the width (stride) of the buffer.
    pub fn stride(&self) -> usize {
        self.dim.width as usize
    }

    /// Gets a slice of the buffer.
    pub fn pixels(&self) -> &[u8] {
        &self.buffer[..]
    }

    /// Gets a mutable slice of the buffer.
    pub fn pixels_mut(&mut self) -> &mut [u8] {
        &mut self.buffer[..]
    }

    /// Set the dimension of the buffer.
    pub fn set_dim(&mut self, dim: TheDim) {
        if dim != self.dim {
            self.dim = dim;
            self.allocate();
        }
    }

    /// Allocates the buffer.
    pub fn allocate(&mut self) {
        if self.dim.is_valid() {
            self.buffer = vec![0; self.dim.width as usize * self.dim.height as usize * 3];
        } else {
            self.buffer = vec![];
        }
    }

    /// Copy the other buffer into this buffer at the given coordinates.
    pub fn copy_into(&mut self, mut x: i32, mut y: i32, other: &TheRGBBuffer) {
        // Early return if the whole other buffer is outside this buffer
        if x + other.dim.width <= 0
            || y + other.dim.height <= 0
            || x >= self.dim.width
            || y >= self.dim.height
        {
            return;
        }

        // Adjust source and destination coordinates and dimensions
        let mut source_offset_x = 0;
        let mut source_y_start = 0;
        let mut copy_width = other.dim.width;
        let mut copy_height = other.dim.height;

        // Adjust for negative x
        if x < 0 {
            source_offset_x = (-x * 3) as usize;
            copy_width += x;
            x = 0;
        }

        // Adjust for negative y
        if y < 0 {
            source_y_start = -y;
            copy_height += y;
            y = 0;
        }

        // Adjust for width overflow
        if x + copy_width > self.dim.width {
            copy_width = self.dim.width - x;
        }

        // Adjust for height overflow
        if y + copy_height > self.dim.height {
            copy_height = self.dim.height - y;
        }

        // Calculate the byte width to copy per row
        let byte_width = (copy_width * 3) as usize;

        // Copy the buffer
        for src_y in source_y_start..source_y_start + copy_height {
            let src_start = (src_y * other.dim.width * 3) as usize + source_offset_x;
            let dst_start = ((src_y + y - source_y_start) * self.dim.width * 3 + x * 3) as usize;

            // Perform the copy
            self.buffer[dst_start..dst_start + byte_width]
                .copy_from_slice(&other.buffer[src_start..src_start + byte_width]);
        }
    }

    /// Returns the pixel at the given UV coordinate as [f32;3]
    pub fn at_f_vec3f(&self, uv: Vec2f) -> Option<Vec3f> {
        let x = (uv.x * self.dim.width as f32) as i32;
        let y = (uv.y * self.dim.height as f32) as i32;

        if x >= 0 && x < self.dim.width && y >= 0 && y < self.dim.height {
            let pixel_index = (y * self.dim.width + x) as usize * 3;
            Some(vec3f(
                (self.buffer[pixel_index] as f32) / 255.0,
                (self.buffer[pixel_index + 1] as f32) / 255.0,
                (self.buffer[pixel_index + 2] as f32) / 255.0,
            ))
        } else {
            None
        }
    }

    /// Returns the pixel at the given UV coordinate.
    pub fn at_f(&self, uv: Vec2f) -> Option<[u8; 3]> {
        let x = (uv.x * self.dim.width as f32) as i32;
        let y = (uv.y * self.dim.height as f32) as i32;

        if x >= 0 && x < self.dim.width && y >= 0 && y < self.dim.height {
            let pixel_index = (y * self.dim.width + x) as usize * 3;
            Some([
                self.buffer[pixel_index],
                self.buffer[pixel_index + 1],
                self.buffer[pixel_index + 2],
            ])
        } else {
            None
        }
    }

    /// Returns the pixel at the given position.
    pub fn at(&self, position: Vec2i) -> Option<[u8; 3]> {
        let x = position.x;
        let y = position.y;

        if x >= 0 && x < self.dim.width && y >= 0 && y < self.dim.height {
            let pixel_index = (y * self.dim.width + x) as usize * 3;
            Some([
                self.buffer[pixel_index],
                self.buffer[pixel_index + 1],
                self.buffer[pixel_index + 2],
            ])
        } else {
            None
        }
    }

    pub fn at_vec3(&self, position: Vec2i) -> Option<Vec3f> {
        let x = position.x;
        let y = position.y;

        if x >= 0 && x < self.dim.width && y >= 0 && y < self.dim.height {
            let pixel_index = (y * self.dim.width + x) as usize * 3;
            Some(vec3f(
                (self.buffer[pixel_index] as f32) / 255.0,
                (self.buffer[pixel_index + 1] as f32) / 255.0,
                (self.buffer[pixel_index + 2] as f32) / 255.0,
            ))
        } else {
            None
        }
    }

    /// Fills the entire buffer with the given RGBA color.
    pub fn fill(&mut self, color: [u8; 3]) {
        for y in 0..self.dim.height {
            for x in 0..self.dim.width {
                let index = (y * self.dim.width + x) as usize * 3;
                // Check to make sure we don't write out of bounds
                if index < self.buffer.len() {
                    self.buffer[index..index + 3].copy_from_slice(&color);
                }
            }
        }
    }

    /// Helper method to calculate the buffer index for a pixel at (x, y).
    pub fn pixel_index(&self, x: i32, y: i32) -> Option<usize> {
        if x >= 0 && x < self.dim.width && y >= 0 && y < self.dim.height {
            Some((y as usize * self.dim.width as usize + x as usize) * 3)
        } else {
            None
        }
    }

    /// Sets the color of a pixel at (x, y).
    pub fn set_pixel(&mut self, x: i32, y: i32, color: &[u8; 3]) {
        if let Some(index) = self.pixel_index(x, y) {
            self.buffer[index..index + 3].copy_from_slice(color);
        }
    }

    /// Sets the color of a pixel at (x, y).
    pub fn set_pixel_vec3f(&mut self, x: i32, y: i32, color: &Vec3f) {
        if let Some(index) = self.pixel_index(x, y) {
            let color = [
                (color.x * 255.0) as u8,
                (color.y * 255.0) as u8,
                (color.z * 255.0) as u8,
            ];
            self.buffer[index..index + 3].copy_from_slice(&color);
        }
    }

    /// Convert the buffer to an RGBA PNG image.
    pub fn to_png(&self) -> Result<Vec<u8>, png::EncodingError> {
        let mut png_data = Vec::new();
        {
            let width = self.dim.width as u32;
            let height = self.dim.height as u32;
            let mut encoder = Encoder::new(&mut png_data, width, height);
            encoder.set_color(ColorType::Rgb);
            encoder.set_depth(BitDepth::Eight);

            let mut writer = encoder.write_header()?;
            writer.write_image_data(&self.buffer)?;
        }
        Ok(png_data)
    }

    /// Copy the buffer to the clipboard.
    pub fn to_clipboard(&self) {
        let img_data = ImageData {
            width: self.dim.width as usize,
            height: self.dim.height as usize,
            bytes: self.buffer.clone().into(),
        };
        if let Ok(mut ctx) = Clipboard::new() {
            ctx.set_image(img_data).unwrap();
        }
    }
}
