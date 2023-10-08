use crate::prelude::*;

pub struct TheContext {
    pub width: usize,
    pub height: usize,

    pub draw: TheDraw2D,
    #[cfg(feature = "renderer")]
    pub renderer: TheRenderer,
    #[cfg(feature = "ui")]
    pub ui: TheUIContext,
}

impl TheContext {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            draw: TheDraw2D::new(),
            #[cfg(feature = "renderer")]
            renderer: TheRenderer::new(),
            #[cfg(feature = "ui")]
            ui: TheUIContext::new(),
        }
    }
}
