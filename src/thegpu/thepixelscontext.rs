use std::sync::Arc;

use pixels::{Pixels, SurfaceTexture};
use winit::window::Window;

use crate::prelude::*;

pub struct ThePixelsContext<'w> {
    pixels: Pixels<'w>,

    width: u32,
    height: u32,
    scale: f32,
}

impl<'w> TheGpuContext for ThePixelsContext<'w> {
    type Error = pixels::Error;
    type Layer = Pixels<'w>;
    type LayerId = usize;
    type ShaderInfo = ();
    type Surface = ();
    type TextureId = usize;

    fn draw(&self) -> Result<Option<Vec<u8>>, Self::Error> {
        self.pixels.render().map(|_| None)
    }

    #[allow(unused)]
    fn layer(&self, layer_id: Self::LayerId) -> Option<&Self::Layer> {
        Some(&self.pixels)
    }

    #[allow(unused)]
    fn layer_mut(&mut self, layer_id: Self::LayerId) -> Option<&mut Self::Layer> {
        Some(&mut self.pixels)
    }

    fn resize(&mut self, width: u32, height: u32) {
        if self.width == width && self.height == height {
            return;
        }

        self.width = width;
        self.height = height;
        // Should panic
        self.pixels.resize_surface(width, height).unwrap();
        // Should panic
        self.pixels
            .resize_buffer(width / self.scale as u32, height / self.scale as u32)
            .unwrap();
    }

    #[allow(unused)]
    fn rotate(&mut self, theta: f32) {
        unimplemented!("Won't support");
    }

    fn scale(&mut self, scale: f32) {
        if self.scale == scale {
            return;
        }

        self.scale = scale;
        // Should panic
        self.pixels
            .resize_buffer(self.width / scale as u32, self.height / scale as u32)
            .unwrap();
    }

    #[allow(unused)]
    fn set_surface(&mut self, width: u32, height: u32, surface: Self::Surface) {
        unimplemented!("Won't support");
    }

    #[allow(unused)]
    fn translate(&mut self, x: f32, y: f32) {
        unimplemented!("Won't support");
    }

    fn translate_coord_to_local(&self, x: u32, y: u32) -> (u32, u32) {
        let (x, y) = self
            .pixels
            .window_pos_to_pixel((x as f32, y as f32))
            .unwrap_or_else(|pos| self.pixels.clamp_pixel_pos(pos));
        (x as u32, y as u32)
    }
}

impl ThePixelsContext<'_> {
    pub fn from_window(window: Arc<Window>) -> Result<Self, pixels::Error> {
        let window_size = window.inner_size();
        let width = window_size.width;
        let height = window_size.height;
        let scale = window.scale_factor() as f32;

        let surface_texture = SurfaceTexture::new(width, height, window);
        let mut pixels = Pixels::new(width, height, surface_texture)?;
        pixels.resize_surface(width, height)?;
        pixels.resize_buffer(width / scale as u32, height / scale as u32)?;

        Ok(Self {
            pixels,
            width,
            height,
            scale,
        })
    }
}
