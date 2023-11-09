use crate::prelude::*;

pub struct TheStackLayout {
    id: TheId,
    dim: TheDim,
    limiter: TheSizeLimiter,

    widgets: Vec<Box<dyn TheWidget>>,
    layouts: Vec<Box<dyn TheLayout>>,
    index: usize,
}

impl TheLayout for TheStackLayout {
    fn new(id: TheId) -> Self
    where
        Self: Sized,
    {
        Self {
            id,
            dim: TheDim::zero(),
            limiter: TheSizeLimiter::new(),

            widgets: vec![],
            layouts: vec![],
            index: 0,
        }
    }

    fn id(&self) -> &TheId {
        &self.id
    }

    fn set_margin(&mut self, margin: Vec4i) {
        if !self.layouts.is_empty() && self.index < self.layouts.len() {
            self.layouts[self.index].set_margin(margin);
        }
    }

    fn set_padding(&mut self, padding: i32) {
        if !self.layouts.is_empty() && self.index < self.layouts.len() {
            self.layouts[self.index].set_padding(padding);
        }
    }

    fn set_background_color(&mut self, color: Option<TheThemeColors>) {
        if !self.layouts.is_empty() && self.index < self.layouts.len() {
            self.layouts[self.index].set_background_color(color);
        }
    }

    fn widgets(&mut self) -> &mut Vec<Box<dyn TheWidget>> {
        if !self.layouts.is_empty() && self.index < self.layouts.len() {
            return self.layouts[self.index].widgets();
        }
        &mut self.widgets
    }

    fn get_widget_at_coord(&mut self, coord: Vec2i) -> Option<&mut Box<dyn TheWidget>> {
        if !self.layouts.is_empty() && self.index < self.layouts.len() {
            return self.layouts[self.index].get_widget_at_coord(coord);
        }
        None
    }

    fn get_layout(
        &mut self,
        name: Option<&String>,
        uuid: Option<&Uuid>,
    ) -> Option<&mut Box<dyn TheLayout>> {
        self.layouts.iter_mut().find(|h| h.id().matches(name, uuid))
    }

    fn get_widget(
        &mut self,
        name: Option<&String>,
        uuid: Option<&Uuid>,
    ) -> Option<&mut Box<dyn TheWidget>> {
        if !self.layouts.is_empty() && self.index < self.layouts.len() {
            return self.layouts[self.index].get_widget(name, uuid);
        }
        None
    }

    fn dim(&self) -> &TheDim {
        if !self.layouts.is_empty() && self.index < self.layouts.len() {
            return self.layouts[self.index].dim();
        }
        &self.dim
    }

    fn dim_mut(&mut self) -> &mut TheDim {
        if !self.layouts.is_empty() && self.index < self.layouts.len() {
            return self.layouts[self.index].dim_mut();
        }
        &mut self.dim
    }

    fn set_dim(&mut self, dim: TheDim, ctx: &mut TheContext) {
        if !self.layouts.is_empty() && self.index < self.layouts.len() {
            self.layouts[self.index].set_dim(dim, ctx);
        }
    }

    fn limiter(&self) -> &TheSizeLimiter {
        if !self.layouts.is_empty() && self.index < self.layouts.len() {
            return self.layouts[self.index].limiter();
        }
        &self.limiter
    }

    fn limiter_mut(&mut self) -> &mut TheSizeLimiter {
        if !self.layouts.is_empty() && self.index < self.layouts.len() {
            return self.layouts[self.index].limiter_mut();
        }
        &mut self.limiter
    }

    fn draw(
        &mut self,
        buffer: &mut TheRGBABuffer,
        style: &mut Box<dyn TheStyle>,
        ctx: &mut TheContext,
    ) {
        if !self.layouts.is_empty() && self.index < self.layouts.len() {
            self.layouts[self.index].draw(buffer, style, ctx);
        }
    }

    /// Convert to the stack layout trait
    fn as_stack_layout(&mut self) -> Option<&mut dyn TheStackLayoutTrait> {
        Some(self)
    }
}

/// TheHLayout specific functions.
pub trait TheStackLayoutTrait: TheLayout {
    /// Add a layout to the stack.
    fn add_layout(&mut self, widget: Box<dyn TheLayout>);

    /// Returns the index of the current layout.
    fn index(&self) -> usize;

    /// Set the index of the current layout.
    fn set_index(&mut self, index: usize);
}

impl TheStackLayoutTrait for TheStackLayout {
    fn add_layout(&mut self, layout: Box<dyn TheLayout>) {
        self.layouts.push(layout);
    }

    fn index(&self) -> usize {
        self.index
    }

    fn set_index(&mut self, index: usize) {
        self.index = index;
    }
}
