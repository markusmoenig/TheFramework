use crate::prelude::*;

pub struct TheColorButton {
    widget_id: TheWidgetId,
    limiter: TheSizeLimiter,

    dim: TheDim,
    color: RGBA,
    is_dirty: bool,
}

impl TheWidget for TheColorButton {
    fn new(name: String) -> Self
    where
        Self: Sized,
    {
        Self {
            widget_id: TheWidgetId::new(name),
            limiter: TheSizeLimiter::new(),

            dim: TheDim::zero(),
            color: WHITE,
            is_dirty: false,
        }
    }

    fn id(&self) -> &TheWidgetId {
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
        let stride = buffer.stride();
        let mut shrinker = TheDimShrinker::zero();

        style.draw_widget_border(buffer, self, &mut shrinker, ctx);

        ctx.draw.rect(
            buffer.pixels_mut(),
            &self.dim.to_buffer_shrunk_utuple(&shrinker),
            stride,
            &self.color,
        );

        if let Some(font) = &ctx.ui.font {
            ctx.draw.text_rect_blend(
                buffer.pixels_mut(),
                &self.dim.to_buffer_shrunk_utuple(&shrinker),
                stride,
                font,
                15.0,
                &self.id().name,
                &BLACK,
                TheHorizontalAlign::Center,
                TheVerticalAlign::Center,
            );
        }

        self.is_dirty = false;
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
