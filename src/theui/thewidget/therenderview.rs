use crate::prelude::*;

pub struct TheRenderView {
    id: TheId,
    limiter: TheSizeLimiter,
    state: TheWidgetState,

    render_buffer: TheRGBABuffer,

    dim: TheDim,

    is_dirty: bool,
}

impl TheWidget for TheRenderView {
    fn new(id: TheId) -> Self
    where
        Self: Sized,
    {
        let mut limiter = TheSizeLimiter::new();
        limiter.set_max_size(vec2i(20, 20));
        Self {
            id,
            limiter,
            state: TheWidgetState::None,

            render_buffer: TheRGBABuffer::new(TheDim::new(0, 0, 20, 20)),

            dim: TheDim::zero(),

            is_dirty: false,
        }
    }

    fn id(&self) -> &TheId {
        &self.id
    }

    #[allow(clippy::single_match)]
    fn on_event(&mut self, event: &TheEvent, ctx: &mut TheContext) -> bool {
        let mut redraw = false;
        // println!("event ({}): {:?}", self.widget_id.name, event);
        match event {
            TheEvent::MouseDown(_coord) => {
                if self.state == TheWidgetState::Selected {
                    self.state = TheWidgetState::None;
                    ctx.ui.send_widget_state_changed(self.id(), self.state);
                } else if self.state != TheWidgetState::Selected {
                    self.state = TheWidgetState::Selected;
                    ctx.ui.send_widget_state_changed(self.id(), self.state);
                }
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

    fn set_state(&mut self, state: TheWidgetState) {
        self.state = state;
        self.is_dirty = true;
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
        _style: &mut Box<dyn TheStyle>,
        _ctx: &mut TheContext,
    ) {
        if !self.dim().is_valid() {
            return;
        }

        self.render_buffer.scaled_into(buffer);

        self.is_dirty = false;
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_render_view(&mut self) -> Option<&mut dyn TheRenderViewTrait> {
        Some(self)
    }
}

pub trait TheRenderViewTrait: TheWidget {
    fn render_buffer_mut(&mut self) -> &mut TheRGBABuffer;
}

impl TheRenderViewTrait for TheRenderView {
    fn render_buffer_mut(&mut self) -> &mut TheRGBABuffer {
        self.is_dirty = true;
        &mut self.render_buffer
    }
}
