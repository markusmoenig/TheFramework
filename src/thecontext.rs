use crate::prelude::*;

pub struct TheContext {
    pub width: usize,
    pub height: usize,
    pub scale_factor: f32,

    pub draw: TheDraw2D,
    #[cfg(feature = "ui")]
    pub ui: TheUIContext,
}

impl TheContext {
    pub fn new(width: usize, height: usize, scale_factor: f32) -> Self {
        Self {
            width,
            height,
            scale_factor,
            draw: TheDraw2D::new(),
            #[cfg(feature = "ui")]
            ui: TheUIContext::new(),
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
