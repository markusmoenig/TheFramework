use crate::prelude::*;

pub struct TheCanvas {
    /// The relative offset to the parent canvas
    pub offset: Vec2i,

    pub dim: TheDim,

    pub limiter: TheSizeLimiter,

    pub root: bool,

    buffer: TheRGBABuffer,

    pub left: Option<Box<TheCanvas>>,
    pub top: Option<Box<TheCanvas>>,
    pub right: Option<Box<TheCanvas>>,
    pub bottom: Option<Box<TheCanvas>>,

    pub widget: Option<Box<dyn TheWidget>>,
}

impl Default for TheCanvas {
    fn default() -> Self {
        Self::new()
    }
}

/// TheCanvas divides a screen dimension into 4 possible sub-spaces for its border while containing a set of widgets for its center.
impl TheCanvas {
    pub fn new() -> Self {
        Self {
            offset: Vec2i::zero(),

            dim: TheDim::zero(),

            limiter: TheSizeLimiter::new(),

            root: false,

            buffer: TheRGBABuffer::empty(),

            left: None,
            top: None,
            right: None,
            bottom: None,

            widget: None,
        }
    }

    /// Set the dimension of the canvas
    pub fn set_dim(&mut self, dim: TheDim) {
        if dim != self.dim {
            self.dim = dim;
            self.buffer.set_dim(self.dim);
            self.layout(self.dim.width, self.dim.height);
        }
    }

    /// Resize the canvas if needed
    pub fn resize(&mut self, width: i32, height: i32) {
        if width != self.dim.width || height != self.dim.height {
            self.set_dim(TheDim::new(self.dim.x, self.dim.y, width, height));
        }
    }

    /// Returns a reference to the underlying buffer
    pub fn buffer(&mut self) -> &TheRGBABuffer {
        &self.buffer
    }

    /// Returns the widget at the given screen coordinate (if any)
    pub fn get_widget(
        &mut self,
        name: Option<&String>,
        uuid: Option<&Uuid>,
    ) -> Option<&mut Box<dyn TheWidget>> {
        if let Some(left) = &mut self.left {
            if let Some(widget) = left.get_widget(name, uuid) {
                return Some(widget);
            }
        }

        if let Some(top) = &mut self.top {
            if let Some(widget) = top.get_widget(name, uuid) {
                return Some(widget);
            }
        }

        if let Some(right) = &mut self.right {
            if let Some(widget) = right.get_widget(name, uuid) {
                return Some(widget);
            }
        }

        if let Some(bottom) = &mut self.bottom {
            if let Some(widget) = bottom.get_widget(name, uuid) {
                return Some(widget);
            }
        }

        if let Some(widget) = &mut self.widget {
            if widget.id().matches(name, uuid) {
                return Some(widget);
            }
        }

        None
    }

    /// Returns the widget at the given screen coordinate (if any)
    pub fn get_widget_at_coord(&mut self, coord: Vec2i) -> Option<&mut Box<dyn TheWidget>> {
        if let Some(left) = &mut self.left {
            if let Some(widget) = left.get_widget_at_coord(coord) {
                return Some(widget);
            }
        }

        if let Some(top) = &mut self.top {
            if let Some(widget) = top.get_widget_at_coord(coord) {
                return Some(widget);
            }
        }

        if let Some(right) = &mut self.right {
            if let Some(widget) = right.get_widget_at_coord(coord) {
                return Some(widget);
            }
        }

        if let Some(bottom) = &mut self.bottom {
            if let Some(widget) = bottom.get_widget_at_coord(coord) {
                return Some(widget);
            }
        }

        if let Some(widget) = &mut self.widget {
            if widget.dim().contains(coord) {
                return Some(widget);
            }
        }

        None
    }

    /// Layout the canvas according to its dimensions.
    pub fn layout(&mut self, width: i32, height: i32) {
        // The screen dimensions
        let mut x = self.dim.x;
        let mut y = self.dim.y;
        let mut w = width;
        let mut h = height;

        // Offset from the buffer
        let mut buffer_x = 0;
        let mut buffer_y = 0;

        if let Some(top) = &mut self.top {
            let top_width = top.limiter.get_width(w);
            let top_height = top.limiter.get_height(h);
            top.set_dim(TheDim::new(x + width - top_width, y, top_width, top_height));
            top.offset = vec2i(0, 0);
            y += top_height;
            buffer_y += top_height;
            h -= top_height;
        }

        if let Some(left) = &mut self.left {
            let left_width = left.limiter.get_width(w);
            let left_height = left.limiter.get_height(h);
            left.set_dim(TheDim::new(x, y, left_width, left_height));
            left.offset = vec2i(0, y);
            x += left_width;
            buffer_x += left_width;
            w -= left_width;
        }

        if let Some(right) = &mut self.right {
            let right_width = right.limiter.get_width(w);
            let right_height = right.limiter.get_height(h);
            right.set_dim(TheDim::new(
                width - right_width,
                y,
                right_width,
                right_height,
            ));
            right.offset = vec2i(width - right_width, y);
            w -= right_width;
        }

        if let Some(bottom) = &mut self.bottom {
            let bottom_width = w;
            let bottom_height = bottom.limiter.get_height(h);
            bottom.set_dim(TheDim::new(
                x,
                y + h - bottom_height,
                bottom_width,
                bottom_height,
            ));
            bottom.offset = vec2i(x, y + h - bottom_height);
            h -= bottom_height;
        }

        if let Some(widget) = &mut self.widget {
            let dim = TheDim::new(x, y, w, h);
            widget.set_dim(dim);
            widget.dim_mut().set_buffer_offset(buffer_x, buffer_y);
        }
    }

    /// Draw the canvas
    pub fn draw(&mut self, style: &mut Box<dyn TheStyle>, ctx: &mut TheContext) {
        if let Some(left) = &mut self.left {
            left.draw(style, ctx);
            self.buffer
                .copy_into(left.offset.x, left.offset.y, &left.buffer);
        }

        if let Some(top) = &mut self.top {
            top.draw(style, ctx);
            self.buffer
                .copy_into(top.offset.x, top.offset.y, &top.buffer);
        }

        if let Some(right) = &mut self.right {
            right.draw(style, ctx);
            self.buffer
                .copy_into(right.offset.x, right.offset.y, &right.buffer);
        }

        if let Some(bottom) = &mut self.bottom {
            bottom.draw(style, ctx);
            self.buffer
                .copy_into(bottom.offset.x, bottom.offset.y, &bottom.buffer);
        }

        if let Some(widget) = &mut self.widget {
            widget.draw(&mut self.buffer, style, ctx);
        }
    }
}
