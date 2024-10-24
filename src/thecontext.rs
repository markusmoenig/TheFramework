use crate::prelude::*;

pub struct TheContext<#[cfg(feature = "gpu_winit")] 'w> {
    pub width: usize,
    pub height: usize,
    pub scale_factor: f32,

    pub draw: TheDraw2D,
    // #[cfg(feature = "renderer")]
    // pub renderer: TheRenderer,
    #[cfg(feature = "ui")]
    pub ui: TheUIContext,
    #[cfg(feature = "gpu_winit")]
    pub gpu: TheGpuContext<'w>,
    #[cfg(feature = "gpu_winit")]
    pub texture_renderer: TheTextureRenderPass,
}

#[cfg(not(feature = "gpu_winit"))]
impl TheContext {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            scale_factor: 1.0,
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

#[cfg(feature = "gpu_winit")]
impl<'w> TheContext<'w> {
    pub fn new(
        width: usize,
        height: usize,
        gpu: TheGpuContext<'w>,
        texture_renderer: TheTextureRenderPass,
    ) -> Self {
        Self {
            width,
            height,
            scale_factor: 1.0,
            draw: TheDraw2D::new(),
            #[cfg(feature = "ui")]
            ui: TheUIContext::new(),
            gpu,
            texture_renderer,
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
