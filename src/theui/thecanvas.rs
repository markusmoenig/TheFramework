use crate::prelude::*;

pub struct TheCanvas {
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

/// TheCanvas divides a screen dimension into 4 possible sub-spaces for its border while containing a set of widgets for its center.
impl TheCanvas {
    pub fn new() -> Self {
        Self {
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
    pub fn get_buffer(&mut self) -> &TheRGBABuffer {
        &self.buffer
    }

    /// Layout the canvas according to its dimensions.
    pub fn layout(&mut self, width: i32, height: i32) {
        let mut x = self.dim.x;
        let mut y = self.dim.y;
        let mut w = width;
        let mut h = height;

        if let Some(top) = &mut self.top {
            let top_width = top.limiter.get_width(w);
            let top_height = top.limiter.get_height(h);
            top.set_dim(TheDim::new(width - top_width, 0, top_width, top_height));
            y += top_height;
            h -= top_height;
        }

        if let Some(left) = &mut self.left {
            let left_width = left.limiter.get_width(w);
            let left_height = left.limiter.get_height(h);
            left.set_dim(TheDim::new(0, y, left_width, left_height));
            x += left_width;
            w -= left_width;
        }

        if let Some(right) = &mut self.right {
            let right_width = right.limiter.get_width(w);
            let right_height = right.limiter.get_height(h);
            right.set_dim(TheDim::new(width - right_width, y, right_width, right_height));
            w -= right_width;
        }

        if let Some(bottom) = &mut self.bottom {
            let bottom_width = w;//top.limiter.get_width(w);
            let bottom_height = bottom.limiter.get_height(h);
            bottom.set_dim(TheDim::new(x, y + h - bottom_height, bottom_width, bottom_height));
            h -= bottom_height;
        }

        if let Some(widget) = &mut self.widget {
            let mut dim = TheDim::new(x, y, w, h);
            dim.root = self.root;
            widget.set_dim(dim);
        }
    }

    /// Draw the canvas
    pub fn draw(&mut self, ctx: &mut TheContext) {

        if let Some(left) = &mut self.left {
            left.draw(ctx);
            self.buffer.copy_into(left.dim.x, left.dim.y, &left.buffer);
        }

        if let Some(top) = &mut self.top {
            top.draw(ctx);
            self.buffer.copy_into(top.dim.x, top.dim.y, &top.buffer);
        }

        if let Some(right) = &mut self.right {
            right.draw(ctx);
            self.buffer.copy_into(right.dim.x, right.dim.y, &right.buffer);
        }

        if let Some(bottom) = &mut self.bottom {
            bottom.draw(ctx);
            self.buffer.copy_into(bottom.dim.x, bottom.dim.y, &bottom.buffer);
        }

        if let Some(widget) = &mut self.widget {
            // println!("{:?}", self.dim.to_utuple());
            widget.draw(&mut self.buffer, ctx);
        }
    }
}
