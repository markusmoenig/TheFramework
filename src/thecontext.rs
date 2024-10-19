use crate::prelude::*;

pub struct TheContext<
    #[cfg(any(feature = "pixels_gpu", feature = "wgpu_gpu"))] 'w,
    #[cfg(feature = "wgpu_gpu")] 's,
> {
    pub width: usize,
    pub height: usize,
    pub scale_factor: f32,

    pub draw: TheDraw2D,
    // #[cfg(feature = "renderer")]
    // pub renderer: TheRenderer,
    #[cfg(feature = "ui")]
    pub ui: TheUIContext,
    #[cfg(feature = "pixels_gpu")]
    pub gpu: ThePixelsContext<'w>,
    #[cfg(feature = "wgpu_gpu")]
    pub gpu: TheWgpuContext<'w, 's>,
}

#[cfg(not(any(feature = "pixels_gpu", feature = "wgpu_gpu")))]
impl TheContext<'_> {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            draw: TheDraw2D::new(),
            // #[cfg(feature = "renderer")]
            // renderer: TheRenderer::new(),
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

#[cfg(feature = "pixels_gpu")]
impl<'w> TheContext<'w> {
    pub fn new(width: usize, height: usize, gpu: ThePixelsContext<'w>) -> Self {
        Self {
            width,
            height,
            draw: TheDraw2D::new(),
            #[cfg(feature = "ui")]
            ui: TheUIContext::new(),
            gpu,
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

#[cfg(feature = "wgpu_gpu")]
impl<'w, 's> TheContext<'w, 's> {
    pub fn new(width: usize, height: usize, gpu: TheWgpuContext<'w, 's>) -> Self {
        Self {
            width,
            height,
            scale_factor: 1.0,
            draw: TheDraw2D::new(),
            #[cfg(feature = "ui")]
            ui: TheUIContext::new(),
            gpu,
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
