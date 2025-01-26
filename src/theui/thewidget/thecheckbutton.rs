use crate::prelude::*;

pub struct TheCheckButton {
    id: TheId,
    limiter: TheSizeLimiter,
    state: TheWidgetState,

    dim: TheDim,
    is_dirty: bool,
}

impl TheWidget for TheCheckButton {
    fn new(id: TheId) -> Self
    where
        Self: Sized,
    {
        let mut limiter = TheSizeLimiter::new();
        limiter.set_max_size(Vec2::new(16, 18));

        Self {
            id,
            limiter,
            state: TheWidgetState::None,

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
                if self.state != TheWidgetState::Selected {
                    self.state = TheWidgetState::Selected;
                } else {
                    self.state = TheWidgetState::None;
                }

                ctx.ui.set_focus(self.id());
                ctx.ui.send_widget_state_changed(self.id(), self.state);
                self.is_dirty = true;
                redraw = true;
            }
            TheEvent::Hover(_coord) => {
                if self.state != TheWidgetState::Selected && !self.id().equals(&ctx.ui.hover) {
                    self.is_dirty = true;
                    ctx.ui.set_hover(self.id());
                    redraw = true;
                }
            }
            // TheEvent::MouseUp(_coord) => {
            //     self.is_dirty = true;
            //     redraw = true;
            // }
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

    fn set_dim(&mut self, dim: TheDim, _ctx: &mut TheContext) {
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

    fn state(&self) -> TheWidgetState {
        self.state
    }

    fn set_state(&mut self, state: TheWidgetState) {
        self.state = state;
        self.is_dirty = true;
    }

    fn supports_hover(&mut self) -> bool {
        true
    }

    fn draw(
        &mut self,
        buffer: &mut TheRGBABuffer,
        _style: &mut Box<dyn TheStyle>,
        ctx: &mut TheContext,
    ) {
        let stride = buffer.stride();

        if !self.dim().is_valid() {
            return;
        }

        let mut icon_name = "dark_checkbutton_normal".to_string();

        if self.id().equals(&ctx.ui.hover) || self.id().equals(&ctx.ui.focus) {
            icon_name = "dark_checkbutton_focus".to_string();
        }

        if self.state == TheWidgetState::Selected {
            icon_name += "_selected";
        }

        if let Some(icon) = ctx.ui.icon(icon_name.as_str()) {
            let utuple = self.dim.to_buffer_utuple();
            let r = (
                (utuple.0 + (utuple.2 - icon.dim().width as usize) / 2),
                utuple.1 + 3,
                icon.dim().width as usize,
                icon.dim().height as usize,
            );
            ctx.draw
                .blend_slice(buffer.pixels_mut(), icon.pixels(), &r, stride);
        }

        self.is_dirty = false;
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
