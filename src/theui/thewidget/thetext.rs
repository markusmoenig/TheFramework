use crate::prelude::*;

pub struct TheText {
    id: TheId,

    limiter: TheSizeLimiter,

    dim: TheDim,
    text: String,
    text_size: f32,

    is_dirty: bool,
}

impl TheWidget for TheText {
    fn new(id: TheId) -> Self
    where
        Self: Sized,
    {
        let mut limiter = TheSizeLimiter::new();
        limiter.set_max_height(21);

        Self {
            id,
            limiter,

            dim: TheDim::zero(),
            text: "".to_string(),
            text_size: 13.0,

            is_dirty: false,
        }
    }

    fn id(&self) -> &TheId {
        &self.id
    }

    // fn on_event(&mut self, event: &TheEvent, ctx: &mut TheContext) -> bool {
    //     false
    // }

    fn dim(&self) -> &TheDim {
        &self.dim
    }

    fn dim_mut(&mut self) -> &mut TheDim {
        &mut self.dim
    }

    fn set_dim(&mut self, dim: TheDim) {
        if self.dim != dim {
            self.dim = dim;
            self.is_dirty = true;
        }
    }

    fn limiter(&self) -> &TheSizeLimiter {
        &self.limiter
    }

    fn limiter_mut(&mut self) -> &mut TheSizeLimiter {
        &mut self.limiter
    }

    fn needs_redraw(&mut self) -> bool {
        self.is_dirty
    }

    fn calculate_size(&mut self, ctx: &mut TheContext) {
        if let Some(font) = &ctx.ui.font {
            let size = ctx.draw.get_text_size(font, self.text_size, &self.text);
            self.limiter_mut()
                .set_max_size(vec2i((ceil(size.0 as f32) + 1.0) as i32, size.1 as i32));
        }
    }

    fn draw(
        &mut self,
        buffer: &mut TheRGBABuffer,
        _style: &mut Box<dyn TheStyle>,
        ctx: &mut TheContext,
    ) {
        if !self.dim().is_valid() {
            return;
        }

        let stride = buffer.stride();

        let mut shrinker = TheDimShrinker::zero();
        shrinker.shrink_by(0, 1, 0, 0);

        if let Some(font) = &ctx.ui.font {
            ctx.draw.text_rect_blend(
                buffer.pixels_mut(),
                &self.dim.to_buffer_shrunk_utuple(&shrinker),
                stride,
                font,
                self.text_size,
                &self.text,
                &WHITE,
                TheHorizontalAlign::Left,
                TheVerticalAlign::Center,
            );
        }

        self.is_dirty = false;
    }
}

/// TheText specific functions.
pub trait TheTextTrait {
    /// Set the text to display.
    fn set_text(&mut self, text: String);
    /// Set the text size.
    fn set_text_size(&mut self, text_size: f32);
}

impl TheTextTrait for TheText {
    fn set_text(&mut self, text: String) {
        self.text = text;
    }
    fn set_text_size(&mut self, text_size: f32) {
        self.text_size = text_size;
    }
}
