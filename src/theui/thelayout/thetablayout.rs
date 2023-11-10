use crate::prelude::*;

pub struct TheTabLayout {
    id: TheId,
    dim: TheDim,
    limiter: TheSizeLimiter,

    tabbar: Box<dyn TheWidget>,

    widgets: Vec<Box<dyn TheWidget>>,
    layouts: Vec<Box<dyn TheLayout>>,
    index: usize,
}

impl TheLayout for TheTabLayout {
    fn new(id: TheId) -> Self
    where
        Self: Sized,
    {
        Self {
            id,
            dim: TheDim::zero(),
            limiter: TheSizeLimiter::new(),

            tabbar: Box::new(TheTabbar::new(TheId::named("Tabbar"))),

            widgets: vec![],
            layouts: vec![],
            index: 0,
        }
    }

    fn id(&self) -> &TheId {
        &self.id
    }

    fn set_margin(&mut self, _margin: Vec4i) {}

    fn widgets(&mut self) -> &mut Vec<Box<dyn TheWidget>> {
        &mut self.widgets
    }

    fn get_widget_at_coord(&mut self, coord: Vec2i) -> Option<&mut Box<dyn TheWidget>> {
        if self.tabbar.dim().contains(coord) {
            return Some(&mut self.tabbar);
        }

        if self.widgets.is_empty() {
            return None;
        }

        let mut index = 0;
        if let Some(tabbar) = self.tabbar.as_tabbar() {
            if let Some(i) = tabbar.selection_index() {
                index = i as usize;
            }
        }

        if index < self.widgets.len() {
            return Some(&mut self.widgets[index]);
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
        if self.tabbar.id().matches(name, uuid) {
            return Some(&mut self.tabbar);
        }

        if self.widgets.is_empty() {
            return None;
        }

        let mut index = 0;
        if let Some(tabbar) = self.tabbar.as_tabbar() {
            if let Some(i) = tabbar.selection_index() {
                index = i as usize;
            }
        }

        if index < self.widgets.len() {
            return Some(&mut self.widgets[index]);
        }

        None
    }

    fn needs_redraw(&mut self) -> bool {
        if self.tabbar.needs_redraw() {
            return true;
        }

        if self.widgets.is_empty() {
            return false;
        }

        let mut index = 0;
        if let Some(tabbar) = self.tabbar.as_tabbar() {
            if let Some(i) = tabbar.selection_index() {
                index = i as usize;
            }
        }

        if index < self.widgets.len() {
            return self.widgets[index].needs_redraw();
        }

        false
    }

    fn dim(&self) -> &TheDim {
        &self.dim
    }

    fn dim_mut(&mut self) -> &mut TheDim {
        &mut self.dim
    }

    fn set_dim(&mut self, dim: TheDim, ctx: &mut TheContext) {
        if self.dim != dim || ctx.ui.relayout {
            self.dim = dim;

            self.tabbar
                .set_dim(TheDim::new(dim.x, dim.y, dim.width, 22));

            self.tabbar
                .dim_mut()
                .set_buffer_offset(self.dim.buffer_x, self.dim.buffer_y);

            for w in &mut self.widgets {
                w.set_dim(TheDim::new(
                    dim.x + 1,
                    dim.y + 23,
                    dim.width - 2,
                    dim.height - 22 - 2,
                ));

                w.dim_mut()
                    .set_buffer_offset(self.dim.buffer_x + 1, self.dim.buffer_y + 23);
            }
        }
    }

    fn limiter(&self) -> &TheSizeLimiter {
        &self.limiter
    }

    fn limiter_mut(&mut self) -> &mut TheSizeLimiter {
        &mut self.limiter
    }

    fn draw(
        &mut self,
        buffer: &mut TheRGBABuffer,
        style: &mut Box<dyn TheStyle>,
        ctx: &mut TheContext,
    ) {
        let stride = buffer.stride();
        let utuple: (usize, usize, usize, usize) = self.dim.to_buffer_utuple();

        ctx.draw.rect(
            buffer.pixels_mut(),
            &utuple,
            stride,
            style.theme().color(TabbarConnector),
        );

        self.tabbar.draw(buffer, style, ctx);

        if self.widgets.is_empty() {
            return;
        }

        let mut index = 0;
        if let Some(tabbar) = self.tabbar.as_tabbar() {
            if let Some(i) = tabbar.selection_index() {
                index = i as usize;
            }
        }

        if index < self.widgets.len() {
            self.widgets[index].draw(buffer, style, ctx);
        }
    }

    /// Convert to the tab layout trait
    fn as_tab_layout(&mut self) -> Option<&mut dyn TheTabLayoutTrait> {
        Some(self)
    }
}

/// TheTabLayoutTrait specific functions.
pub trait TheTabLayoutTrait: TheLayout {
    /// Add a layout to the stack.
    fn add_widget(&mut self, name: String, widget: Box<dyn TheWidget>);

    /// Returns the index of the current layout.
    fn index(&self) -> usize;

    /// Set the index of the current layout.
    fn set_index(&mut self, index: usize);
}

impl TheTabLayoutTrait for TheTabLayout {
    fn add_widget(&mut self, name: String, widget: Box<dyn TheWidget>) {
        if let Some(tabbar) = self.tabbar.as_tabbar() {
            tabbar.add_tab(name);
        }
        self.widgets.push(widget);
    }

    fn index(&self) -> usize {
        self.index
    }

    fn set_index(&mut self, index: usize) {
        self.index = index;
    }
}
