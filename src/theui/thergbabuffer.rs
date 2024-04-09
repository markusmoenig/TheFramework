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

    /// Blends the other buffer into this buffer at the given coordinates.
    pub fn blend_into(&mut self, x: i32, y: i32, other: &TheRGBABuffer) {
        let width = other.dim.width as usize;

        let stride = self.stride();
        let dest = &mut self.buffer[..];

        for h in 0..other.dim.height {
            for w in 0..width {
                let dest_x = w as i32 + x;
                let dest_y = h + y;

                // Check if the destination coordinates are within the bounds of the destination buffer
                if dest_x >= 0 && dest_x < self.dim.width && dest_y >= 0 && dest_y < self.dim.height
                {
                    let src_index = (h as usize * width + w) * 4;
                    let dst_index = (dest_y as usize * stride + dest_x as usize) * 4;

                    if dst_index + 3 < dest.len() && src_index + 3 < other.buffer.len() {
                        let src_pixel = &other.buffer[src_index..src_index + 4];
                        let dst_pixel = &mut dest[dst_index..dst_index + 4];

                        // Alpha blending
                        let alpha = src_pixel[3] as f32 / 255.0;
                        let inv_alpha = 1.0 - alpha;

                        dst_pixel[0] =
                            (alpha * src_pixel[0] as f32 + inv_alpha * dst_pixel[0] as f32) as u8;
                        dst_pixel[1] =
                            (alpha * src_pixel[1] as f32 + inv_alpha * dst_pixel[1] as f32) as u8;
                        dst_pixel[2] =
                            (alpha * src_pixel[2] as f32 + inv_alpha * dst_pixel[2] as f32) as u8;
                        // Optionally blend the alpha itself
                        // dst_pixel[3] = (alpha * src_pixel[3] as f32 + inv_alpha * dst_pixel[3] as f32) as u8;
                    }
                }
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

    /// Renders text using a fondue::Font.
    pub fn draw_text(&mut self, font: &fontdue::Font, text: &str, size: f32, color: [u8; 4]) {
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

        let halign = TheHorizontalAlign::Center;
        let valign = TheVerticalAlign::Center;

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
            //println!("Metrics: {:?}", glyph);

            for y in 0..metrics.height {
                for x in 0..metrics.width {
                    // if (y + rect.1) >= rect.1
                    //     && (y + rect.1) < (rect.1 + rect.3)
                    //     && (x + rect.0) >= rect.0
                    //     && (x + rect.0) < (rect.0 + rect.2)
                    // {

                    // let i = (x + glyph.x as usize) * 4
                    //     + (y + glyph.y as usize) * stride * 4;
                    let m = alphamap[x + y * metrics.width];

                    if let Some(index) =
                        self.pixel_index(x as i32 + glyph.x as i32, y as i32 + glyph.y as i32)
                    {
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
