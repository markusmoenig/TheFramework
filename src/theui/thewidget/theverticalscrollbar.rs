use crate::prelude::*;

pub struct TheVerticalScrollbar {
    widget_id: TheId,
    limiter: TheSizeLimiter,

    state: TheWidgetState,

    scroll_offset: i32,
    total_height: i32,

    mouse_down_coord: Vec2i,

    dim: TheDim,
    is_dirty: bool,
}

impl TheWidget for TheVerticalScrollbar {
    fn new(name: String) -> Self
    where
        Self: Sized,
    {
        let mut limiter = TheSizeLimiter::new();
        limiter.set_max_width(142);
        limiter.set_max_height(20);

        Self {
            widget_id: TheId::new(name),
            limiter,

            state: TheWidgetState::None,

            scroll_offset: 0,
            total_height: 0,

            mouse_down_coord: Vec2i::zero(),

            dim: TheDim::zero(),
            is_dirty: false,
        }
    }

    fn id(&self) -> &TheId {
        &self.widget_id
    }

    fn on_event(&mut self, event: &TheEvent, ctx: &mut TheContext) -> bool {
        let mut redraw = false;
        // println!("event ({}): {:?}", self.widget_id.name, event);
        match event {
            TheEvent::MouseDown(coord) => {
                self.is_dirty = true;
                if self.state != TheWidgetState::Clicked {
                    self.state = TheWidgetState::Clicked;
                    ctx.ui.send_widget_state_changed(self.id(), self.state);
                    ctx.ui.set_focus(self.id());
                }
                if let Some(coord) = coord.to_vec2i() {
                    self.mouse_down_coord = coord;
                }
                redraw = true;
            }
            TheEvent::MouseDragged(coord) => {
                self.is_dirty = true;
                redraw = true;
                if let Some(coord) = coord.to_vec2i() {
                    let d = coord - self.mouse_down_coord;
                    self.scroll_by(d.y);
                    self.mouse_down_coord = coord;
                }
            }
            TheEvent::MouseUp(_coord) => {
                self.is_dirty = true;
                if self.state == TheWidgetState::Clicked {
                    self.state = TheWidgetState::None;
                    ctx.ui.send_widget_state_changed(self.id(), self.state);
                }
                redraw = true;
            }
            TheEvent::Hover(_coord) => {
                if self.state != TheWidgetState::Clicked && !self.id().equals(&ctx.ui.hover) {
                    self.is_dirty = true;
                    ctx.ui.set_hover(self.id());
                    redraw = true;
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
        style: &mut Box<dyn TheStyle>,
        ctx: &mut TheContext,
    ) {
        if !self.dim().is_valid() {
            return;
        }

        let stride = buffer.stride();
        //let mut shrinker = TheDimShrinker::zero();

        let utuple: (usize, usize, usize, usize) = self.dim.to_buffer_utuple();

        ctx.draw.rect(
            buffer.pixels_mut(),
            &utuple,
            stride,
            style.theme().color(ScrollbarBackground),
        );

        let mut icon_name = if self.state == TheWidgetState::Clicked {
            "dark_vertical_scrollbar_clicked_".to_string()
        } else {
            "dark_vertical_scrollbar_normal_".to_string()
        };

        if self.state != TheWidgetState::Clicked && self.id().equals(&ctx.ui.hover) {
            icon_name = "dark_vertical_scrollbar_hover_".to_string()
        }

        let scroll_bar_height = (self.dim.height as f32 * self.scrollbar_thumb_proportional_height()).floor() as i32;
        let offset = (self.dim.height as f32 * self.scrollbar_position()) as usize; //self.scroll_offset as usize;

        println!("{} {}", offset, self.scrollbar_position());

        if scroll_bar_height >= 5 {
            if let Some(icon) = ctx.ui.icon(&(icon_name.clone() + "top")) {
                let r = (
                    utuple.0,
                    utuple.1 + offset,
                    icon.dim().width as usize,
                    icon.dim().height as usize,
                );
                ctx.draw
                    .blend_slice(buffer.pixels_mut(), icon.pixels(), &r, stride);
            }
        }

        if scroll_bar_height > 10 {
            if let Some(icon) = ctx.ui.icon(&(icon_name.clone() + "middle")) {
                let mut r = (
                    utuple.0,
                    utuple.1 + offset + 5,
                    icon.dim().width as usize,
                    icon.dim().height as usize,
                );
                for _ in 0..scroll_bar_height - 10 {
                    ctx.draw
                        .blend_slice(buffer.pixels_mut(), icon.pixels(), &r, stride);
                    r.1 += 1;
                }
            }
        }

        if scroll_bar_height >= 10 {
            if let Some(icon) = ctx.ui.icon(&(icon_name + "bottom")) {
                let r = (
                    utuple.0,
                    utuple.1 + offset + scroll_bar_height as usize - 5,
                    icon.dim().width as usize,
                    icon.dim().height as usize,
                );
                ctx.draw
                    .blend_slice(buffer.pixels_mut(), icon.pixels(), &r, stride);
            }
        }

        self.is_dirty = false;
    }

    fn as_vertical_scrollbar(&mut self) -> Option<&mut dyn TheVerticalScrollbarTrait> {
        Some(self)
    }
}

pub trait TheVerticalScrollbarTrait {
    /// Returns the total height of the content.
    fn total_height(&self) -> i32;

    /// Sets the total heigh of the content.
    fn set_total_height(&mut self, value: i32);

    /// Returns the visible height of the widget.
    fn viewport_height(&self) -> i32;

    /// Returns the current vertical scroll offset.
    fn scroll_offset(&self) -> i32;

    /// Sets the vertical scroll offset.
    fn set_scroll_offset(&mut self, offset: i32);

    /// Helper function to scroll by a certain amount (delta).
    /// This can be positive (to scroll down) or negative (to scroll up).
    fn scroll_by(&mut self, delta: i32) {
        let new_offset = (self.scroll_offset() + delta)
            .max(0)
            .min(self.total_height() - self.viewport_height());
        self.set_scroll_offset(new_offset);
    }

    /// Helper function to determine if the scrollbar is needed.
    fn needs_scrollbar(&self) -> bool;

    /// Get the position of the scrollbar slider as a ratio in the range 0 to 1.
    /// Useful for drawing the scrollbar thumb.
    fn scrollbar_position(&self) -> f32 {
        if self.needs_scrollbar() {
            (self.scroll_offset() as f32 + 0.0) / (self.total_height() - self.viewport_height()) as f32
        } else {
            0.0
        }
    }

    /// Get the height of the scrollbar slider (thumb) as a proportion of the viewport height.
    fn scrollbar_thumb_proportional_height(&self) -> f32 {
        if self.needs_scrollbar() {
            self.viewport_height() as f32 / self.total_height() as f32
        } else {
            1.0
        }
    }
}

impl TheVerticalScrollbarTrait for TheVerticalScrollbar {
    fn total_height(&self) -> i32 {
        self.total_height
    }

    fn set_total_height(&mut self, value: i32) {
        self.total_height = value;
    }

    fn viewport_height(&self) -> i32 {
        self.dim().height
    }

    fn scroll_offset(&self) -> i32 {
        self.scroll_offset
    }

    fn set_scroll_offset(&mut self, offset: i32) {
        self.scroll_offset = offset;
    }

    fn needs_scrollbar(&self) -> bool {
        self.total_height() > self.dim().height
    }
}
