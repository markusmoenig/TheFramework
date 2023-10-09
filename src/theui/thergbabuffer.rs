use crate::prelude::*;

pub struct TheRGBABuffer {
    dim: TheDim,

    buffer: Vec<u8>,
}

/// TheRGBABuffer contains the pixel buffer for a canvas.
impl TheRGBABuffer {
    /// Create an empty buffer.
    pub fn empty() -> Self {
        Self {
            dim: TheDim::zero(),
            buffer: vec![],
        }
    }

    /// Creates a buffer of the given dimension
    pub fn new(dim: TheDim) -> Self {
        Self {
            dim: dim,
            buffer: vec![0; dim.width as usize * dim.height as usize * 4],
        }
    }

    /// Gets a mutable slice of the buffer
    pub fn get_stride(&self) -> usize {
        self.dim.width as usize
    }

    /// Gets a slice of the buffer
    pub fn get(&self) -> &[u8] {
        &self.buffer[..]
    }

    /// Gets a mutable slice of the buffer
    pub fn get_mut(&mut self) -> &mut [u8] {
        &mut self.buffer[..]
    }

    /// Set the dimension of the buffer
    pub fn set_dim(&mut self, dim: TheDim) {
        if dim != self.dim {
            self.dim = dim;
            self.allocate();
        }
    }

    /// Allocates the buffer
    pub fn allocate(&mut self) {
        if self.dim.is_valid() {
            self.buffer = vec![0; self.dim.width as usize * self.dim.height as usize * 4];
        } else {
            self.buffer = vec![];
        }
    }

    pub fn copy_into(&mut self, x: i32, y: i32, other: &TheRGBABuffer) {
        let dest = &mut self.buffer[..];
        let width = (other.dim.width * 4) as usize;
        for h in 0..other.dim.height  {
            let s = (h * other.dim.width * 4) as usize;
            let d = ((h+y) * self.dim.width * 4 + x * 4) as usize;
            dest[d..d + width].copy_from_slice(&other.buffer[s..s + width]);
        }
    }
}
