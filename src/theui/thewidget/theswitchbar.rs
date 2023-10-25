use crate::prelude::*;

pub struct TheSwitchbar {
    widget_id: TheId,

    limiter: TheSizeLimiter,

    dim: TheDim,
    text: String,
    is_dirty: bool,
}

impl TheWidget for TheSwitchbar {
    fn new(name: String) -> Self
    where
        Self: Sized,
    {
        let mut limiter = TheSizeLimiter::new();
        limiter.set_max_height(21);

        Self {
            widget_id: TheId::new(name),
            limiter,

            dim: TheDim::zero(),
            text: "".to_string(),
            is_dirty: false,
        }
    }

    fn id(&self) -> &TheId {
        &self.widget_id
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

    fn draw(
        &mut self,
        buffer: &mut TheRGBABuffer,
        style: &mut Box<dyn TheStyle>,
        ctx: &mut TheContext,
    ) {
        if !self.dim().is_valid() {
            return;
        }

        let stride = buffer.stride();
        let utuple: (usize, usize, usize, usize) = self.dim.to_buffer_utuple();

        ctx.draw.rect_outline(
            buffer.pixels_mut(),
            &utuple,
            stride,
            style.theme().color(SwitchbarBorder),
        );

        if let Some(icon) = ctx.ui.icon("dark_switchbar") {
            for x in 1..utuple.2 - 1 {
                let r = (utuple.0 + x, utuple.1, 1, icon.dim().height as usize);
                ctx.draw
                    .copy_slice_3(buffer.pixels_mut(), icon.pixels(), &r, stride);
            }
        }

        if let Some(icon) = ctx.ui.icon("switchbar_icon") {
            let r = (
                utuple.0 + 6,
                utuple.1 + 6,
                icon.dim().width as usize,
                icon.dim().height as usize,
            );
            ctx.draw
                .blend_slice(buffer.pixels_mut(), icon.pixels(), &r, stride);
        }

        let mut shrinker = TheDimShrinker::zero();
        shrinker.shrink_by(30, 1, 0, 0);

        if let Some(font) = &ctx.ui.font {
            ctx.draw.text_rect_blend(
                buffer.pixels_mut(),
                &self.dim.to_buffer_shrunk_utuple(&shrinker),
                stride,
                font,
                15.0,
                &self.id().name,
                &WHITE,
                TheHorizontalAlign::Left,
                TheVerticalAlign::Center,
            );
        }

        self.is_dirty = false;
    }
}

pub trait TheSwitchbarTrait {
    fn set_text(&mut self, text: String);
}

impl TheSwitchbarTrait for TheSwitchbar {
    fn set_text(&mut self, text: String) {
        self.text = text;
    }
}
