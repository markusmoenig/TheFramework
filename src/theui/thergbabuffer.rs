use super::{compress, decompress};
use crate::prelude::*;
use arboard::{Clipboard, ImageData};
use fontdue::layout::{
    CoordinateSystem, HorizontalAlign, Layout, LayoutSettings, TextStyle, VerticalAlign,
};
use png::{BitDepth, ColorType, Encoder};
use std::ops::{Index, IndexMut, Range};

#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Clone, Debug)]
pub struct TheRGBABuffer {
    dim: TheDim,

    #[serde(serialize_with = "compress", deserialize_with = "decompress")]
    buffer: Vec<u8>,
}

impl Default for TheRGBABuffer {
    fn default() -> Self {
        Self::empty()
    }
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

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        !self.is_valid()
    }

    /// Returns the len of the underlying Vec<u8>
    pub fn len(&self) -> usize {
        self.dim.width as usize * self.dim.height as usize * 4
    }

    /// Allocates the buffer.
    pub fn allocate(&mut self) {
        if self.dim.is_valid() {
            self.buffer = vec![0; self.dim.width as usize * self.dim.height as usize * 4];
        } else {
            self.buffer = vec![];
        }
    }

    /// Extracts a sub-buffer of given dimensions from the current buffer.
    pub fn extract(&self, dim: &TheDim) -> Self {
        let mut new_buffer = Self::new(*dim);

        for y in 0..dim.height {
            for x in 0..dim.width {
                let src_x = dim.x + x;
                let src_y = dim.y + y;

                if src_x >= 0 && src_x < self.dim.width && src_y >= 0 && src_y < self.dim.height {
                    let src_index = ((src_y * self.dim.width) + src_x) as usize * 4;
                    let dest_index = ((y * dim.width) + x) as usize * 4;

                    if src_index + 3 < self.buffer.len() && dest_index + 3 < new_buffer.buffer.len()
                    {
                        new_buffer.buffer[dest_index..dest_index + 4]
                            .copy_from_slice(&self.buffer[src_index..src_index + 4]);
                    }
                }
            }
        }

        new_buffer
    }

    /// Copy the other buffer into this buffer at the given coordinates.
    pub fn copy_into(&mut self, mut x: i32, mut y: i32, other: &TheRGBABuffer) {
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
            source_offset_x = (-x * 4) as usize;
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
        let byte_width = (copy_width * 4) as usize;

        // Copy the buffer
        for src_y in source_y_start..source_y_start + copy_height {
            let src_start = (src_y * other.dim.width * 4) as usize + source_offset_x;
            let dst_start = ((src_y + y - source_y_start) * self.dim.width * 4 + x * 4) as usize;

            // Perform the copy
            self.buffer[dst_start..dst_start + byte_width]
                .copy_from_slice(&other.buffer[src_start..src_start + byte_width]);
        }
    }

    /// Blend the other buffer into this buffer at the given coordinates.
    pub fn blend_into(&mut self, mut x: i32, mut y: i32, other: &TheRGBABuffer) {
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
            source_offset_x = (-x * 4) as usize;
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

        // Blend the buffer
        for src_y in source_y_start..source_y_start + copy_height {
            let src_start = (src_y * other.dim.width * 4) as usize + source_offset_x;
            let dst_start = ((src_y + y - source_y_start) * self.dim.width * 4 + x * 4) as usize;

            for i in 0..copy_width {
                let src_idx = src_start + i as usize * 4;
                let dst_idx = dst_start + i as usize * 4;

                let src_pixel = &other.buffer[src_idx..src_idx + 4];
                let dst_pixel = &mut self.buffer[dst_idx..dst_idx + 4];

                let src_alpha = src_pixel[3] as f32 / 255.0;
                let inv_alpha = 1.0 - src_alpha;

                for j in 0..3 {
                    dst_pixel[j] =
                        (src_pixel[j] as f32 * src_alpha + dst_pixel[j] as f32 * inv_alpha) as u8;
                }

                // Update alpha if necessary (e.g., for premultiplied alpha)
                dst_pixel[3] =
                    (src_pixel[3] as f32 * src_alpha + dst_pixel[3] as f32 * inv_alpha) as u8;
            }
        }
    }

    /// Copy the horizontal range of the other buffer into this buffer at the given coordinates.
    pub fn copy_horizontal_range_into(
        &mut self,
        x: i32,
        y: i32,
        other: &TheRGBABuffer,
        range: Range<i32>,
    ) {
        let dest = &mut self.buffer[..];
        let height = other.dim.height as usize;
        let stride = self.dim.width * 4;

        for (dw, w) in range.enumerate() {
            if w >= other.dim.width {
                break;
            }
            let s_start = (w * 4) as usize;
            let d_start = ((x + dw as i32) * 4) as usize;

            for h in 0..height {
                let s = s_start + h * other.dim().width as usize * 4;
                let d = d_start + ((y + h as i32) * stride) as usize;
                dest[d..d + 4].copy_from_slice(&other.buffer[s..s + 4]);
            }
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

    /// Creates a scaled version of the buffer by writing into the other buffer.
    pub fn scaled_into(&self, into: &mut TheRGBABuffer) {
        let new_width = into.dim().width;
        let new_height = into.dim().height;

        let scale_x = new_width as f32 / self.dim.width as f32;
        let scale_y = new_height as f32 / self.dim.height as f32;

        for y in 0..new_height {
            for x in 0..new_width {
                let src_x = (x as f32 / scale_x).round() as i32;
                let src_y = (y as f32 / scale_y).round() as i32;

                let pixel_index = (src_y * self.dim.width + src_x) as usize * 4;
                let new_pixel_index = (y * new_width + x) as usize * 4;

                if pixel_index < self.buffer.len() && new_pixel_index < into.buffer.len() {
                    into.buffer[new_pixel_index..new_pixel_index + 4]
                        .copy_from_slice(&self.buffer[pixel_index..pixel_index + 4]);
                }
            }
        }
    }

    /// Creates a scaled version of the buffer by writing into the other buffer.
    pub fn scaled_into_linear(&self, into: &mut TheRGBABuffer) {
        let new_width = into.dim().width;
        let new_height = into.dim().height;

        let scale_x = self.dim.width as f32 / new_width as f32;
        let scale_y = self.dim.height as f32 / new_height as f32;

        for y in 0..new_height {
            for x in 0..new_width {
                let src_x = x as f32 * scale_x;
                let src_y = y as f32 * scale_y;

                let src_x0 = src_x.floor() as i32;
                let src_y0 = src_y.floor() as i32;
                let src_x1 = (src_x0 + 1).min(self.dim.width - 1);
                let src_y1 = (src_y0 + 1).min(self.dim.height - 1);

                let t_x = src_x - src_x0 as f32;
                let t_y = src_y - src_y0 as f32;

                let pixel_index00 = (src_y0 * self.dim.width + src_x0) as usize * 4;
                let pixel_index10 = (src_y0 * self.dim.width + src_x1) as usize * 4;
                let pixel_index01 = (src_y1 * self.dim.width + src_x0) as usize * 4;
                let pixel_index11 = (src_y1 * self.dim.width + src_x1) as usize * 4;

                let new_pixel_index = (y * new_width + x) as usize * 4;

                for i in 0..4 {
                    let v00 = self.buffer[pixel_index00 + i] as f32;
                    let v10 = self.buffer[pixel_index10 + i] as f32;
                    let v01 = self.buffer[pixel_index01 + i] as f32;
                    let v11 = self.buffer[pixel_index11 + i] as f32;

                    let v0 = v00 + (v10 - v00) * t_x;
                    let v1 = v01 + (v11 - v01) * t_x;
                    let v = v0 + (v1 - v0) * t_y;

                    into.buffer[new_pixel_index + i] = v.round() as u8;
                }
            }
        }
    }

    /// Creates a scaled version of the buffer by writing into the other buffer while respecting the dimensions.
    pub fn scaled_into_using_dim(&self, into: &mut TheRGBABuffer, dim: &TheDim) {
        let new_width = dim.width;
        let new_height = dim.height;

        let scale_x = new_width as f32 / self.dim.width as f32;
        let scale_y = new_height as f32 / self.dim.height as f32;

        for y in 0..new_height {
            for x in 0..new_width {
                let src_x = (x as f32 / scale_x).round() as i32;
                let src_y = (y as f32 / scale_y).round() as i32;

                let pixel_index = (src_y * self.dim.width + src_x) as usize * 4;
                let new_pixel_index =
                    ((y + dim.buffer_y) * into.stride() as i32 + x + dim.buffer_x) as usize * 4;

                if pixel_index < self.buffer.len() && new_pixel_index < into.buffer.len() {
                    into.buffer[new_pixel_index..new_pixel_index + 4]
                        .copy_from_slice(&self.buffer[pixel_index..pixel_index + 4]);
                }
            }
        }
    }

    /// Extracts a region from the buffer.
    pub fn extract_region(&self, region: &TheRGBARegion) -> TheRGBABuffer {
        let mut tile_buffer =
            TheRGBABuffer::new(TheDim::new(0, 0, region.width as i32, region.height as i32));

        for y in 0..region.height as i32 {
            for x in 0..region.width as i32 {
                let buffer_index = ((self.dim.y + region.y as i32 + y) * self.dim.width
                    + self.dim.x
                    + region.x as i32
                    + x) as usize
                    * 4;
                let tile_index = (y * region.width as i32 + x) as usize * 4;

                if buffer_index < self.buffer.len() && tile_index < tile_buffer.buffer.len() {
                    tile_buffer.buffer[tile_index..tile_index + 4]
                        .copy_from_slice(&self.buffer[buffer_index..buffer_index + 4]);
                }
            }
        }

        tile_buffer
    }

    /// Extracts the regions of the sequence from the buffer.
    pub fn extract_sequence(&self, sequence: &TheRGBARegionSequence) -> Vec<TheRGBABuffer> {
        sequence
            .regions
            .iter()
            .map(|region| self.extract_region(region))
            .collect()
    }

    /// Returns the pixel at the given UV coordinate as [f32;4]
    pub fn at_f_vec4f(&self, uv: Vec2f) -> Option<Vec4f> {
        let x = (uv.x * self.dim.width as f32) as i32;
        let y = (uv.y * self.dim.height as f32) as i32;

        self.pixel_index(x, y).map(|pixel_index| {
            vec4f(
                (self.buffer[pixel_index] as f32) / 255.0,
                (self.buffer[pixel_index + 1] as f32) / 255.0,
                (self.buffer[pixel_index + 2] as f32) / 255.0,
                (self.buffer[pixel_index + 3] as f32) / 255.0,
            )
        })
    }

    /// Returns the pixel at the given UV coordinate.
    pub fn at_f(&self, uv: Vec2f) -> Option<[u8; 4]> {
        let x = (uv.x * self.dim.width as f32) as i32;
        let y = (uv.y * self.dim.height as f32) as i32;

        if x >= 0 && x < self.dim.width && y >= 0 && y < self.dim.height {
            let pixel_index = (y * self.dim.width + x) as usize * 4;
            Some([
                self.buffer[pixel_index],
                self.buffer[pixel_index + 1],
                self.buffer[pixel_index + 2],
                self.buffer[pixel_index + 3],
            ])
        } else {
            None
        }
    }

    /// Returns the pixel at the given position.
    pub fn at(&self, position: Vec2i) -> Option<[u8; 4]> {
        let x = position.x;
        let y = position.y;

        if x >= 0 && x < self.dim.width && y >= 0 && y < self.dim.height {
            let pixel_index = (y * self.dim.width + x) as usize * 4;
            Some([
                self.buffer[pixel_index],
                self.buffer[pixel_index + 1],
                self.buffer[pixel_index + 2],
                self.buffer[pixel_index + 3],
            ])
        } else {
            None
        }
    }

    pub fn at_vec4(&self, position: Vec2i) -> Option<Vec4f> {
        let x = position.x;
        let y = position.y;

        if x >= 0 && x < self.dim.width && y >= 0 && y < self.dim.height {
            let pixel_index = (y * self.dim.width + x) as usize * 4;
            Some(vec4f(
                (self.buffer[pixel_index] as f32) / 255.0,
                (self.buffer[pixel_index + 1] as f32) / 255.0,
                (self.buffer[pixel_index + 2] as f32) / 255.0,
                (self.buffer[pixel_index + 3] as f32) / 255.0,
            ))
        } else {
            None
        }
    }

    /// Fills the entire buffer with the given RGBA color.
    pub fn fill(&mut self, color: [u8; 4]) {
        for y in 0..self.dim.height {
            for x in 0..self.dim.width {
                let index = (y * self.dim.width + x) as usize * 4;
                // Check to make sure we don't write out of bounds
                if index < self.buffer.len() {
                    self.buffer[index..index + 4].copy_from_slice(&color);
                }
            }
        }
    }

    /// Draws a line from (x0, y0) to (x1, y1) with the given color.
    pub fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: [u8; 4]) {
        let mut x = x0;
        let mut y = y0;
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy; // Error value e_xy

        loop {
            // Set pixel color
            if let Some(pixel_index) = self.pixel_index(x, y) {
                self.buffer[pixel_index..pixel_index + 4].copy_from_slice(&color);
            }

            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy; // e_xy+e_x > 0
                x += sx;
            }
            if e2 <= dx {
                err += dx; // e_xy+e_y < 0
                y += sy;
            }
        }
    }

    /// Draws a horizontal line from (x0, y) to (x1, y) with the given color.
    pub fn draw_horizontal_line(&mut self, x0: i32, x1: i32, y: i32, color: [u8; 4]) {
        let mut start_x = x0.min(x1);
        let end_x = x0.max(x1);

        // Ensure that the line is within bounds
        if y < 0 || y >= self.dim.height || start_x >= self.dim.width {
            return;
        }

        // Clip start_x to buffer bounds
        if start_x < 0 {
            start_x = 0;
        }

        for x in start_x..=end_x {
            if x < self.dim.width {
                self.set_pixel(x, y, &color);
            } else {
                break;
            }
        }
    }

    /// Draws a vertical line from (x, y0) to (x, y1) with the given color.
    pub fn draw_vertical_line(&mut self, x: i32, y0: i32, y1: i32, color: [u8; 4]) {
        let mut start_y = y0.min(y1);
        let end_y = y0.max(y1);

        // Ensure that the line is within bounds
        if x < 0 || x >= self.dim.width || start_y >= self.dim.height {
            return;
        }

        // Clip start_y to buffer bounds
        if start_y < 0 {
            start_y = 0;
        }

        for y in start_y..=end_y {
            if y < self.dim.height {
                self.set_pixel(x, y, &color);
            } else {
                break;
            }
        }
    }

    /// Draws the outline of a given rectangle
    pub fn draw_rect_outline(&mut self, rect: &TheDim, color: &[u8; 4]) {
        let y = rect.y;
        for x in rect.x..rect.x + rect.width {
            self.set_pixel(x, y, color);
            self.set_pixel(x, y + rect.height - 1, color);
        }

        let x = rect.x;
        for y in rect.y..rect.y + rect.height {
            self.set_pixel(x, y, color);
            self.set_pixel(x + rect.width - 1, y, color);
        }
    }

    /// Draws a rounded rect with a border
    pub fn draw_rounded_rect(
        &mut self,
        dim: &TheDim,
        color: &[u8; 4],
        rounding: &(f32, f32, f32, f32),
        border_size: f32,
        border_color: &[u8; 4],
    ) {
        let hb = border_size / 2.0;
        let center = (
            (dim.x as f32 + dim.width as f32 / 2.0 - hb).round(),
            (dim.y as f32 + dim.height as f32 / 2.0 - hb).round(),
        );
        for y in dim.y..dim.y + dim.height {
            for x in dim.x..dim.x + dim.width {
                let p = (x as f32 - center.0, y as f32 - center.1);
                let mut r: (f32, f32);

                if p.0 > 0.0 {
                    r = (rounding.0, rounding.1);
                } else {
                    r = (rounding.2, rounding.3);
                }

                if p.1 <= 0.0 {
                    r.0 = r.1;
                }

                let q: (f32, f32) = (
                    p.0.abs() - dim.width as f32 / 2.0 + hb + r.0,
                    p.1.abs() - dim.height as f32 / 2.0 + hb + r.0,
                );
                let d = f32::min(f32::max(q.0, q.1), 0.0)
                    + self.length((f32::max(q.0, 0.0), f32::max(q.1, 0.0)))
                    - r.0;

                if d < 1.0 {
                    let t = self.fill_mask(d);

                    if let Some(background) = self.at(vec2i(x, y)) {
                        let mut mixed_color =
                            self.mix_color(&background, color, t * (color[3] as f32 / 255.0));

                        let b = self.border_mask(d, border_size);
                        mixed_color = self.mix_color(&mixed_color, border_color, b);

                        self.set_pixel(x, y, &mixed_color)
                    }
                }
            }
        }
    }

    /// Draws a rounded rect with a border
    pub fn draw_disc(
        &mut self,
        dim: &TheDim,
        color: &[u8; 4],
        border_size: f32,
        border_color: &[u8; 4],
    ) {
        let hb = border_size / 2.0;
        let center = (
            (dim.x as f32 + dim.width as f32 / 2.0 - hb).round(),
            (dim.y as f32 + dim.height as f32 / 2.0 - hb).round(),
        );
        for y in dim.y..dim.y + dim.height {
            for x in dim.x..dim.x + dim.width {
                let p = vec2f(x as f32 - center.0, y as f32 - center.1);
                let r = dim.width as f32 / 2.0 - hb;

                let d = length(p) - r;

                if d < 1.0 {
                    let t = self.fill_mask(d);

                    if let Some(background) = self.at(vec2i(x, y)) {
                        let mut mixed_color =
                            self.mix_color(&background, color, t * (color[3] as f32 / 255.0));

                        let b = self.border_mask(d, border_size);
                        mixed_color = self.mix_color(&mixed_color, border_color, b);

                        self.set_pixel(x, y, &mixed_color)
                    }
                }
            }
        }
    }

    /// The fill mask for an SDF distance
    fn fill_mask(&self, dist: f32) -> f32 {
        (-dist).clamp(0.0, 1.0)
    }

    /// The border mask for an SDF distance
    fn border_mask(&self, dist: f32, width: f32) -> f32 {
        (dist + width).clamp(0.0, 1.0) - dist.clamp(0.0, 1.0)
    }

    /// Mixes two colors based on v
    fn mix_color(&self, a: &[u8; 4], b: &[u8; 4], v: f32) -> [u8; 4] {
        [
            (((1.0 - v) * (a[0] as f32 / 255.0) + b[0] as f32 / 255.0 * v) * 255.0) as u8,
            (((1.0 - v) * (a[1] as f32 / 255.0) + b[1] as f32 / 255.0 * v) * 255.0) as u8,
            (((1.0 - v) * (a[2] as f32 / 255.0) + b[2] as f32 / 255.0 * v) * 255.0) as u8,
            (((1.0 - v) * (a[3] as f32 / 255.0) + b[3] as f32 / 255.0 * v) * 255.0) as u8,
        ]
    }

    // Length of a 2d vector
    fn length(&self, v: (f32, f32)) -> f32 {
        ((v.0).powf(2.0) + (v.1).powf(2.0)).sqrt()
    }

    #[allow(clippy::too_many_arguments)]
    /// Render an aligned text in the buffer.
    pub fn draw_text(
        &mut self,
        position: Vec2i,
        font: &fontdue::Font,
        text: &str,
        size: f32,
        color: [u8; 4],
        halign: TheHorizontalAlign,
        valign: TheVerticalAlign,
    ) {
        pub fn mix_color(a: &[u8; 4], b: &[u8; 4], v: f32) -> [u8; 4] {
            [
                (((1.0 - v) * (a[0] as f32 / 255.0) + b[0] as f32 / 255.0 * v) * 255.0) as u8,
                (((1.0 - v) * (a[1] as f32 / 255.0) + b[1] as f32 / 255.0 * v) * 255.0) as u8,
                (((1.0 - v) * (a[2] as f32 / 255.0) + b[2] as f32 / 255.0 * v) * 255.0) as u8,
                (((1.0 - v) * (a[3] as f32 / 255.0) + b[3] as f32 / 255.0 * v) * 255.0) as u8,
            ]
        }

        // fn get_text_size(font: &Font, size: f32, text: &str) -> (usize, usize) {
        //     let fonts = &[font];

        //     let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        //     layout.reset(&LayoutSettings {
        //         ..LayoutSettings::default()
        //     });
        //     layout.append(fonts, &TextStyle::new(text, size, 0));

        //     let x = layout.glyphs()[layout.glyphs().len() - 1].x.ceil() as usize
        //         + layout.glyphs()[layout.glyphs().len() - 1].width
        //         + 1;
        //     (x, layout.height() as usize)
        // }

        // let (width, height) = get_text_size(font, size, text);

        let fonts = &[font];

        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.reset(&LayoutSettings {
            max_width: Some(self.dim.width as f32),
            max_height: Some(self.dim.height as f32),
            horizontal_align: if halign == TheHorizontalAlign::Left {
                HorizontalAlign::Left
            } else if halign == TheHorizontalAlign::Right {
                HorizontalAlign::Right
            } else {
                HorizontalAlign::Center
            },
            vertical_align: if valign == TheVerticalAlign::Top {
                VerticalAlign::Top
            } else if valign == TheVerticalAlign::Bottom {
                VerticalAlign::Bottom
            } else {
                VerticalAlign::Middle
            },
            ..LayoutSettings::default()
        });
        layout.append(fonts, &TextStyle::new(text, size, 0));

        for glyph in layout.glyphs() {
            let (metrics, alphamap) = font.rasterize(glyph.parent, glyph.key.px);
            for y in 0..metrics.height {
                for x in 0..metrics.width {
                    let m = alphamap[x + y * metrics.width];

                    if let Some(index) = self.pixel_index(
                        x as i32 + glyph.x as i32 + position.x,
                        y as i32 + glyph.y as i32 + position.y,
                    ) {
                        let background = &[
                            self.buffer[index],
                            self.buffer[index + 1],
                            self.buffer[index + 2],
                            self.buffer[index + 3],
                        ];
                        self.buffer[index..index + 4].copy_from_slice(&mix_color(
                            background,
                            &color,
                            m as f32 / 255.0,
                        ));
                    }
                }
            }
        }
    }

    /// Helper method to calculate the buffer index for a pixel at (x, y).
    pub fn pixel_index(&self, x: i32, y: i32) -> Option<usize> {
        if x >= 0 && x < self.dim.width && y >= 0 && y < self.dim.height {
            Some((y as usize * self.dim.width as usize + x as usize) * 4)
        } else {
            None
        }
    }

    /// Get a pixel at (x, y).
    pub fn get_pixel(&self, x: i32, y: i32) -> Option<[u8; 4]> {
        self.pixel_index(x, y).map(|index| {
            [
                self.buffer[index],
                self.buffer[index + 1],
                self.buffer[index + 2],
                self.buffer[index + 3],
            ]
        })
    }

    /// Sets the color of a pixel at (x, y).
    pub fn set_pixel(&mut self, x: i32, y: i32, color: &[u8; 4]) {
        if let Some(index) = self.pixel_index(x, y) {
            self.buffer[index..index + 4].copy_from_slice(color);
        }
    }

    /// Convert the buffer to an RGBA PNG image.
    pub fn to_png(&self) -> Result<Vec<u8>, png::EncodingError> {
        let mut png_data = Vec::new();
        {
            let width = self.dim.width as u32;
            let height = self.dim.height as u32;
            let mut encoder = Encoder::new(&mut png_data, width, height);
            encoder.set_color(ColorType::Rgba);
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
        let mut region_buffer =
            TheRGBABuffer::new(TheDim::new(0, 0, self.width as i32, self.height as i32));
        for y in 0..self.height as i32 {
            for x in 0..self.width as i32 {
                let buffer_index =
                    ((self.y as i32 + y) * buffer.dim().width + self.x as i32 + x) as usize * 4;
                let region_index = (y * self.width as i32 + x) as usize * 4;

                if buffer_index < buffer.pixels().len()
                    && region_index < region_buffer.pixels_mut().len()
                {
                    region_buffer.pixels_mut()[region_index..region_index + 4]
                        .copy_from_slice(&buffer.pixels()[buffer_index..buffer_index + 4]);
                }
            }
        }

        // Scale the extracted region
        region_buffer.scaled(new_width, new_height)
    }
}

#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Clone, Debug)]
pub struct TheRGBARegionSequence {
    pub regions: Vec<TheRGBARegion>,
}

impl Default for TheRGBARegionSequence {
    fn default() -> Self {
        Self::new()
    }
}

/// TheRGBARegionSequence holds an array of RGBA regions, used to identify a tile.
impl TheRGBARegionSequence {
    pub fn new() -> Self {
        Self { regions: vec![] }
    }
}

// Implement Index and IndexMut
impl Index<usize> for TheRGBARegionSequence {
    type Output = TheRGBARegion;

    fn index(&self, index: usize) -> &Self::Output {
        &self.regions[index]
    }
}

impl IndexMut<usize> for TheRGBARegionSequence {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.regions[index]
    }
}

#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Clone, Debug)]
pub struct TheRGBATile {
    pub id: Uuid,
    pub name: String,
    pub buffer: Vec<TheRGBABuffer>,
    pub role: u8,
    pub blocking: bool,
    pub billboard: bool,
}

impl Default for TheRGBATile {
    fn default() -> Self {
        Self::new()
    }
}

/// TheRGBARegionSequence holds an array of RGBA regions, used to identify a tile.
impl TheRGBATile {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: String::default(),
            buffer: vec![],
            role: 0,
            blocking: false,
            billboard: false,
        }
    }

    pub fn buffer(buffer: TheRGBABuffer) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: String::default(),
            buffer: vec![buffer],
            role: 0,
            blocking: false,
            billboard: false,
        }
    }
}
