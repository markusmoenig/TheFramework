use crate::prelude::*;

pub struct TheCanvas {
    pub uuid: Uuid,

    /// The relative offset to the parent canvas
    pub offset: Vec2i,

    pub dim: TheDim,

    pub root: bool,
    pub top_is_expanding: bool,

    buffer: TheRGBABuffer,

    left: Option<Box<TheCanvas>>,
    top: Option<Box<TheCanvas>>,
    right: Option<Box<TheCanvas>>,
    bottom: Option<Box<TheCanvas>>,

    widget: Option<Box<dyn TheWidget>>,
    layout: Option<Box<dyn TheLayout>>,
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
            uuid: Uuid::new_v4(),

            offset: Vec2i::zero(),

            dim: TheDim::zero(),

            root: false,
            top_is_expanding: true,

            buffer: TheRGBABuffer::empty(),

            left: None,
            top: None,
            right: None,
            bottom: None,

            widget: None,
            layout: None,
        }
    }

    /// Set the dimension of the canvas
    pub fn set_dim(&mut self, dim: TheDim, ctx: &mut TheContext) {
        if dim != self.dim || ctx.ui.relayout {
            self.dim = dim;
            self.buffer.set_dim(self.dim);
            self.layout(self.dim.width, self.dim.height, ctx);
        }
    }

    /// Returns a reference to the limiter of the widget.
    fn _limiter(&self) -> Option<&TheSizeLimiter> {
        if let Some(widget) = &self.widget {
            return Some(widget.limiter());
        } else if let Some(layout) = &self.layout {
            return Some(layout.limiter());
        }
        None
    }

    /// Returns a mutable reference to the limiter of the widget.
    fn _limiter_mut(&mut self) -> Option<&mut TheSizeLimiter> {
        if let Some(widget) = &mut self.widget {
            return Some(widget.limiter_mut());
        } else if let Some(layout) = &mut self.layout {
            return Some(layout.limiter_mut());
        }
        None
    }

    /// Returns the width of the limiter considering the maximum width of the widget.
    fn get_limiter_width(&self, max_width: i32) -> i32 {
        if let Some(widget) = &self.widget {
            return widget.limiter().get_width(max_width);
        } else if let Some(layout) = &self.layout {
            return layout.limiter().get_width(max_width);
        }
        max_width
    }

    /// Returns the height of the limiter considering the given maximum height.
    fn get_limiter_height(&self, max_height: i32) -> i32 {
        if let Some(widget) = &self.widget {
            return widget.limiter().get_height(max_height);
        } else if let Some(layout) = &self.layout {
            return layout.limiter().get_height(max_height);
        }
        max_height
    }

    /// Sets the widget.
    pub fn set_widget<T: TheWidget + 'static>(&mut self, widget: T) {
        self.widget = Some(Box::new(widget));
    }

    /// Sets the layout.
    pub fn set_layout<T: TheLayout + 'static>(&mut self, layout: T) {
        self.layout = Some(Box::new(layout));
    }

    /// Sets the canvas to the left of this canvas.
    pub fn set_left(&mut self, canvas: TheCanvas) {
        self.left = Some(Box::new(canvas));
    }

    /// Sets the canvas to the top of this canvas.
    pub fn set_top(&mut self, canvas: TheCanvas) {
        self.top = Some(Box::new(canvas));
    }

    /// Sets the canvas to the right of this canvas.
    pub fn set_right(&mut self, canvas: TheCanvas) {
        self.right = Some(Box::new(canvas));
    }

    /// Sets the canvas to the bottom of this canvas.
    pub fn set_bottom(&mut self, canvas: TheCanvas) {
        self.bottom = Some(Box::new(canvas));
    }

    /// Resize the canvas if needed
    pub fn resize(&mut self, width: i32, height: i32, ctx: &mut TheContext) {
        if width != self.dim.width || height != self.dim.height {
            self.set_dim(TheDim::new(self.dim.x, self.dim.y, width, height), ctx);
        }
    }

    /// Returns a reference to the underlying buffer
    pub fn buffer(&mut self) -> &TheRGBABuffer {
        &self.buffer
    }

    /// Returns the canvas of the given id
    pub fn get_canvas(&mut self, uuid: Uuid) -> Option<&mut TheCanvas> {
        if uuid == self.uuid {
            return Some(self);
        }

        if let Some(left) = &mut self.left {
            if let Some(canvas) = left.get_canvas(uuid) {
                return Some(canvas);
            }
        }

        if let Some(top) = &mut self.top {
            if let Some(canvas) = top.get_canvas(uuid) {
                return Some(canvas);
            }
        }

        if let Some(right) = &mut self.right {
            if let Some(canvas) = right.get_canvas(uuid) {
                return Some(canvas);
            }
        }

        if let Some(bottom) = &mut self.bottom {
            if let Some(canvas) = bottom.get_canvas(uuid) {
                return Some(canvas);
            }
        }

        None
    }

    /// Returns the widget of the given id
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

        if let Some(layout) = &mut self.layout {
            if let Some(child) = layout.get_widget(name, uuid) {
                return Some(child);
            }
        }

        if let Some(widget) = &mut self.widget {
            if widget.id().matches(name, uuid) {
                return Some(widget);
            }
        }

        None
    }

    /// Returns the layout of the given id
    pub fn get_layout(
        &mut self,
        name: Option<&String>,
        uuid: Option<&Uuid>,
    ) -> Option<&mut Box<dyn TheLayout>> {
        if let Some(left) = &mut self.left {
            if let Some(layout) = left.get_layout(name, uuid) {
                return Some(layout);
            }
        }

        if let Some(top) = &mut self.top {
            if let Some(layout) = top.get_layout(name, uuid) {
                return Some(layout);
            }
        }

        if let Some(right) = &mut self.right {
            if let Some(layout) = right.get_layout(name, uuid) {
                return Some(layout);
            }
        }

        if let Some(bottom) = &mut self.bottom {
            if let Some(layout) = bottom.get_layout(name, uuid) {
                return Some(layout);
            }
        }

        if let Some(layout) = &mut self.layout {
            if layout.id().matches(name, uuid) {
                return Some(layout);
            }

            if let Some(layout) = layout.as_stack_layout() {
                if let Some(layout) = layout.get_layout(name, uuid) {
                    return Some(layout);
                }
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

        if let Some(layout) = &mut self.layout {
            if let Some(widget) = layout.get_widget_at_coord(coord) {
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
    pub fn layout(&mut self, width: i32, height: i32, ctx: &mut TheContext) {
        // The screen dimensions
        let mut x = self.dim.x;
        let mut y = self.dim.y;
        let mut w = width;
        let mut h = height;

        // Offset from the buffer
        let mut buffer_x = 0;
        let mut buffer_y = 0;

        if self.top_is_expanding {
            if let Some(top) = &mut self.top {
                let top_width = top.get_limiter_width(w);
                let top_height = top.get_limiter_height(h);
                top.set_dim(
                    TheDim::new(x + width - top_width, y, top_width, top_height),
                    ctx,
                );
                top.offset = vec2i(0, 0);
                y += top_height;
                buffer_y += top_height;
                h -= top_height;
            }
        }

        if let Some(left) = &mut self.left {
            let left_width = left.get_limiter_width(w);
            let left_height = left.get_limiter_height(h);
            left.set_dim(TheDim::new(x, y, left_width, left_height), ctx);
            left.offset = vec2i(0, buffer_y);
            x += left_width;
            buffer_x += left_width;
            w -= left_width;
        }

        let mut right_width = 0;
        if let Some(right) = &mut self.right {
            right_width = right.get_limiter_width(w);
            let right_height = right.get_limiter_height(h);
            right.set_dim(
                TheDim::new(x + w - right_width, y, right_width, right_height),
                ctx,
            );
            right.offset = vec2i(width - right_width, buffer_y);
            w -= right_width;
        }

        if !self.top_is_expanding {
            if let Some(top) = &mut self.top {
                let top_width = top.get_limiter_width(w);
                let top_height = top.get_limiter_height(h);
                top.set_dim(
                    TheDim::new(
                        x + width - top_width - right_width,
                        y,
                        top_width,
                        top_height,
                    ),
                    ctx,
                );
                top.offset = vec2i(0, 0);
                y += top_height;
                buffer_y += top_height;
                h -= top_height;
            }
        }

        if let Some(bottom) = &mut self.bottom {
            let bottom_width = w;
            let bottom_height = bottom.get_limiter_height(h);
            bottom.set_dim(
                TheDim::new(x, y + h - bottom_height, bottom_width, bottom_height),
                ctx,
            );
            bottom.offset = vec2i(buffer_x, buffer_y + h - bottom_height);
            h -= bottom_height;
        }

        if let Some(widget) = &mut self.widget {
            let dim = TheDim::new(x, y, w, h);
            widget.set_dim(dim);
            widget.dim_mut().set_buffer_offset(buffer_x, buffer_y);
        }

        if let Some(layout) = &mut self.layout {
            let mut dim = TheDim::new(x, y, w, h);
            dim.buffer_x = buffer_x;
            dim.buffer_y = buffer_y;
            layout.set_dim(dim, ctx);
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

        // If a layout needs a redraw, make sure to redraw the widget as well as items in the layout may be transparent (text)

        let mut force_widget_redraw = false;

        if let Some(layout) = &mut self.layout {
            force_widget_redraw = layout.needs_redraw();
        }

        if let Some(widget) = &mut self.widget {
            // println!("drawing widget {}, {:?}", widget.id().name, widget.dim());
            if ctx.ui.redraw_all || widget.needs_redraw() || force_widget_redraw {
                widget.draw(&mut self.buffer, style, ctx);
            }
        }

        if let Some(layout) = &mut self.layout {
            // println!("drawing layout {}, {:?}", layout.id().name, layout.dim());
            if ctx.ui.redraw_all || layout.needs_redraw() || layout.widgets().is_empty() {
                layout.draw(&mut self.buffer, style, ctx);
            }
        }
    }

    pub fn draw_overlay(&mut self, style: &mut Box<dyn TheStyle>, ctx: &mut TheContext) {
        if let Some(overlay) = &ctx.ui.overlay {
            if let Some(widget) = self.get_widget(None, Some(&overlay.uuid)) {
                let buffer = widget.draw_overlay(style, ctx);
                if buffer.is_valid() {
                    self.buffer
                        .copy_into(buffer.dim().x, buffer.dim().y, &buffer);
                }
            }
        }
    }
}
