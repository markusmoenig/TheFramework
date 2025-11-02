use std::{num::NonZeroU32, sync::Arc};

use softbuffer::Surface;
use winit::window::Window;

use crate::prelude::*;

pub struct TheContext {
    pub width: usize,
    pub height: usize,
    pub scale_factor: f32,

    pub draw: TheDraw2D,
    #[cfg(feature = "ui")]
    pub ui: TheUIContext,
    pub surface: Surface<Arc<Window>, Arc<Window>>,
}

impl TheContext {
    pub fn new(width: usize, height: usize, scale_factor: f32, window: Arc<Window>) -> Self {
        let context = softbuffer::Context::new(window.clone()).unwrap();
        let mut surface = softbuffer::Surface::new(&context, window.clone()).unwrap();
        surface
            .resize(
                NonZeroU32::new(width as u32 * scale_factor as u32).unwrap(),
                NonZeroU32::new(height as u32 * scale_factor as u32).unwrap(),
            )
            .unwrap();

        Self {
            width,
            height,
            scale_factor,
            draw: TheDraw2D::new(),
            #[cfg(feature = "ui")]
            ui: TheUIContext::new(),
            surface,
        }
    }

    /// Gets the current time in milliseconds.
    pub fn get_time(&self) -> u128 {
        let time;
        #[cfg(not(target_arch = "wasm32"))]
        {
            use std::time::{SystemTime, UNIX_EPOCH};
            let t = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards");
            time = t.as_millis();
        }
        #[cfg(target_arch = "wasm32")]
        {
            time = web_sys::window().unwrap().performance().unwrap().now() as u128;
        }
        time
    }
}
