use crate::prelude::*;

pub struct TheColorButton {
    dim: TheDim,

    color: RGBA
}

impl TheWidget for TheColorButton {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            dim: TheDim::zero(),
            color : WHITE
        }
    }

    /// Set the dimension of the widget
     fn set_dim(&mut self, dim: TheDim) {
        self.dim = dim;
    }

    fn draw(&mut self, buffer: &mut TheRGBABuffer, ctx: &mut TheContext) {
        let stride = buffer.get_stride();
        ctx.draw.rect(
            buffer.get_mut(),
            &self.dim.to_utuple(),
            stride,
            &self.color,
        );
    }
}

pub trait TheColorColorButtonTrait {
    fn set_color(&mut self, color: RGBA);
}

impl TheColorColorButtonTrait for TheColorButton {
    fn set_color(&mut self, color: RGBA) {
        self.color = color;
    }
}
