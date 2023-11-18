use crate::prelude::*;
use std::ops::Range;

use super::{compress, decompress};

#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Clone, Debug)]
pub struct TheRGBABuffer {
    dim: TheDim,

    #[serde(serialize_with = "compress", deserialize_with = "decompress")]
    buffer: Vec<u8>,
}

/// TheRGBABuffer contains the pixel buffer for a canvas or icon.
impl TheRGBABuffer {
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
            buffer: vec![0; dim.width as usize * dim.height as usize * 4],
        }
    }

    /// Creates a buffer from existing data.
    pub fn from(buffer: Vec<u8>, width: u32, height: u32) -> Self {
        Self {
            dim: TheDim::new(0, 0, width as i32, height as i32),
            buffer,
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
            self.buffer = vec![0; self.dim.width as usize * self.dim.height as usize * 4];
        } else {
            self.buffer = vec![];
        }
    }

    /// Copy the other buffer into this buffer at the given coordinates.
    pub fn copy_into(&mut self, x: i32, y: i32, other: &TheRGBABuffer) {
        let dest = &mut self.buffer[..];
        let width = (other.dim.width * 4) as usize;
        for h in 0..other.dim.height {
            let s = (h * other.dim.width * 4) as usize;
            let d = ((h + y) * self.dim.width * 4 + x * 4) as usize;
            dest[d..d + width].copy_from_slice(&other.buffer[s..s + width]);
        }
    }

    /// Copy the vertical range of the other buffer into this buffer at the given coordinates.
    pub fn copy_vertical_range_into(
        &mut self,
        x: i32,
        y: i32,
        other: &TheRGBABuffer,
        range: Range<i32>,
    ) {
        let dest = &mut self.buffer[..];
        let width = (other.dim.width * 4) as usize;

        for (dh, h) in range.enumerate() {
            if h >= other.dim.height {
                break;
            }
            let s = (h * other.dim.width * 4) as usize;
            let d = ((dh as i32 + y) * self.dim.width * 4 + x * 4) as usize;
            dest[d..d + width].copy_from_slice(&other.buffer[s..s + width]);
        }
    }

    /// Creates a scaled version of the buffer.
    pub fn scaled(&self, new_width: i32, new_height: i32) -> Self {
        let scale_x = new_width as f32 / self.dim.width as f32;
        let scale_y = new_height as f32 / self.dim.height as f32;

        let mut new_buffer = TheRGBABuffer::new(TheDim::new(0, 0, new_width, new_height));

        for y in 0..new_height {
            for x in 0..new_width {
                let src_x = (x as f32 / scale_x).round() as i32;
                let src_y = (y as f32 / scale_y).round() as i32;

                let pixel_index = (src_y * self.dim.width + src_x) as usize * 4;
                let new_pixel_index = (y * new_width + x) as usize * 4;

                if pixel_index < self.buffer.len() && new_pixel_index < new_buffer.buffer.len() {
                    new_buffer.buffer[new_pixel_index..new_pixel_index + 4]
                        .copy_from_slice(&self.buffer[pixel_index..pixel_index + 4]);
                }
            }
        }

        new_buffer
    }
}

#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Clone, Debug)]
pub struct TheRGBARegion {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

/// TheRGBARegion points to a rectangular region in TheRGBABuffer. Used for tile management.
impl TheRGBARegion {
    /// Creates a new region of the given dimension.
    pub fn new(x: usize, y: usize, width: usize, height: usize) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Scales the region of the buffer to a new width and height.
    pub fn scale(&self, buffer: &TheRGBABuffer, new_width: i32, new_height: i32) -> TheRGBABuffer {
        // Extract the region from the buffer
        let mut region_buffer = TheRGBABuffer::new(TheDim::new(0, 0, self.width as i32, self.height as i32));
        for y in 0..self.height as i32 {
            for x in 0..self.width as i32 {
                let buffer_index = ((self.y as i32 + y) * buffer.dim().width + self.x as i32 + x) as usize * 4;
                let region_index = (y * self.width as i32 + x) as usize * 4;

                if buffer_index < buffer.pixels().len() && region_index < region_buffer.pixels_mut().len() {
                    region_buffer.pixels_mut()[region_index..region_index + 4]
                        .copy_from_slice(&buffer.pixels()[buffer_index..buffer_index + 4]);
                }
            }
        }

        // Scale the extracted region
        region_buffer.scaled(new_width, new_height)
    }
}
