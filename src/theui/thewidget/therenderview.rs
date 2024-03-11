use crate::prelude::*;

pub struct TheRenderView {
    id: TheId,
    limiter: TheSizeLimiter,
    state: TheWidgetState,

    render_buffer: TheRGBABuffer,
    wheel_scale: f32,
    accumulated_wheel_delta: Vec2f,

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
            wheel_scale: -0.4,
            accumulated_wheel_delta: Vec2f::zero(),

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
            TheEvent::MouseDown(coord) => {
                if self.state == TheWidgetState::Selected {
                    self.state = TheWidgetState::None;
                    ctx.ui.send_widget_state_changed(self.id(), self.state);
                } else if self.state != TheWidgetState::Selected {
                    self.state = TheWidgetState::Selected;
                    ctx.ui.send_widget_state_changed(self.id(), self.state);
                }
                ctx.ui.set_focus(self.id());

                ctx.ui
                    .send(TheEvent::RenderViewClicked(self.id().clone(), *coord));

                self.is_dirty = true;
                redraw = true;
            }
            TheEvent::Hover(_coord) => {
                if !self.id().equals(&ctx.ui.hover) {
                    self.is_dirty = true;
                    ctx.ui.set_hover(self.id());
                    redraw = true;
                }
            }
            TheEvent::MouseWheel(delta) => {
                let scale_factor = self.wheel_scale; // * 1.0 / (self.zoom.powf(0.5));

                let aspect_ratio = self.dim().width as f32 / self.dim().height as f32;

                let scale_x = if aspect_ratio > 1.0 {
                    1.0 / aspect_ratio
                } else {
                    1.0
                };
                let scale_y = if aspect_ratio < 1.0 {
                    aspect_ratio
                } else {
                    1.0
                };

                // Update accumulated deltas
                self.accumulated_wheel_delta.x += delta.x as f32 * scale_factor * scale_x;
                self.accumulated_wheel_delta.y += delta.y as f32 * scale_factor * scale_y;

                let minimum_delta_threshold = 2.0;

                // Check if accumulated deltas exceed the threshold
                if self.accumulated_wheel_delta.x.abs() > minimum_delta_threshold
                    || self.accumulated_wheel_delta.y.abs() > minimum_delta_threshold
                {
                    // Convert accumulated deltas to integer and reset
                    let d = vec2i(
                        self.accumulated_wheel_delta.x as i32,
                        self.accumulated_wheel_delta.y as i32,
                    );
                    self.accumulated_wheel_delta = Vec2f::zero();

                    ctx.ui
                        .send(TheEvent::RenderViewScrollBy(self.id().clone(), d));
                }
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
        style: &mut Box<dyn TheStyle>,
        ctx: &mut TheContext,
    ) {
        if !self.dim().is_valid() {
            return;
        }

        self.render_buffer.scaled_into(buffer);

        let stride = buffer.stride();
        if Some(self.id.clone()) == ctx.ui.focus {
            let tuple = self.dim().to_buffer_utuple();
            ctx.draw.rect_outline(
                buffer.pixels_mut(),
                &tuple,
                stride,
                style.theme().color(DefaultSelection),
            );
        }
        self.is_dirty = false;
    }

    fn supports_hover(&mut self) -> bool {
        true
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
