use crate::prelude::*;

pub struct TheContext {
    pub width           : usize,
    pub height          : usize,

    pub draw            : TheDraw2D,
    pub renderer        : TheRenderer,
}

impl TheContext {

    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            draw        : TheDraw2D::new(),
            renderer    : TheRenderer::new(),
        }
    }
}