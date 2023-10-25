use crate::prelude::*;

pub struct TheTextLineEdit {
    widget_id: TheId,
    limiter: TheSizeLimiter,

    text: String,

    dim: TheDim,
    is_dirty: bool,
}

impl TheWidget for TheTextLineEdit {
    fn new(name: String) -> Self
    where
        Self: Sized,
    {
        let mut limiter = TheSizeLimiter::new();
        limiter.set_max_height(20);

        Self {
            widget_id: TheId::new(name),
            limiter,

            text: "".to_string(),

            dim: TheDim::zero(),
            is_dirty: false,
        }
    }

    fn id(&self) -> &TheId {
        &self.widget_id
    }

    #[allow(clippy::single_match)]
    fn on_event(&mut self, event: &TheEvent, ctx: &mut TheContext) -> bool {
        let mut redraw = false;
        // println!("event ({}): {:?}", self.widget_id.name, event);
        match event {
            TheEvent::MouseDown(_coord) => {
                ctx.ui.set_focus(self.id());
                self.is_dirty = true;
                redraw = true;
            }
            _ => {}
        }
        redraw
    }

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

    fn set_needs_redraw(&mut self, redraw: bool) {
        self.is_dirty = redraw;
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
        let mut shrinker = TheDimShrinker::zero();

        style.draw_widget_border(buffer, self, &mut shrinker, ctx);

        ctx.draw.rect(
            buffer.pixels_mut(),
            &self.dim.to_buffer_shrunk_utuple(&shrinker),
            stride,
            style.theme().color(TextEditBackground),
        );

        if let Some(font) = &ctx.ui.font {
            ctx.draw.text_rect_blend(
                buffer.pixels_mut(),
                &self.dim.to_buffer_shrunk_utuple(&shrinker),
                stride,
                font,
                14.5,
                &self.text,
                style.theme().color(TextEditTextColor),
                TheHorizontalAlign::Left,
                TheVerticalAlign::Center,
            );
        }

        self.is_dirty = false;
    }
}

pub trait TheTextLineEditTrait: TheWidget {
    fn set_text(&mut self, text: String);
}

impl TheTextLineEditTrait for TheTextLineEdit {
    fn set_text(&mut self, text: String) {
        self.text = text;
    }
}
