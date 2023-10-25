use crate::prelude::*;

pub struct TheSnapperLayout {
    widget_id: TheWidgetId,
    limiter: TheSizeLimiter,

    dim: TheDim,

    bars: Vec<Box<dyn TheWidget>>,

    layouts: Vec<Box<dyn TheLayout>>,
    widgets: Vec<Box<dyn TheWidget>>,

    margin: Vec4i,

    background: Option<TheThemeColors>,
}

impl TheLayout for TheSnapperLayout {
    fn new(name: String) -> Self
    where
        Self: Sized,
    {
        Self {
            widget_id: TheWidgetId::new(name),
            limiter: TheSizeLimiter::new(),

            dim: TheDim::zero(),

            bars: vec![],

            layouts: vec![],
            widgets: vec![],

            margin: vec4i(0, 0, 0, 0),

            background: Some(TextLayoutBackground),
        }
    }

    fn id(&self) -> &TheWidgetId {
        &self.widget_id
    }

    fn set_margin(&mut self, margin: Vec4i) {
        self.margin = margin;
    }

    fn set_background_color(&mut self, color: Option<TheThemeColors>) {
        self.background = color;
    }

    fn widgets(&mut self) -> &mut Vec<Box<dyn TheWidget>> {
        &mut self.widgets
    }

    fn get_widget_at_coord(&mut self, coord: Vec2i) -> Option<&mut Box<dyn TheWidget>> {
        for b in &mut self.bars {
            if b.dim().contains(coord) {
                return Some(b);
            }
        }
        for l in &mut self.layouts {
            if let Some(w) = l.get_widget_at_coord(coord) {
                return Some(w);
            }
        }
        None
    }

    fn get_widget(
        &mut self,
        name: Option<&String>,
        uuid: Option<&Uuid>,
    ) -> Option<&mut Box<dyn TheWidget>> {

        if let Some(w) = self.bars.iter_mut().find(|b| b.id().matches(name, uuid)) {
            return Some(w);
        }

        for l in &mut self.layouts {
            let widgets = l.widgets();
            if let Some(w) = widgets.iter_mut().find(|w| w.id().matches(name, uuid)) {
                return Some(w);
            }
        }

        None
    }

    fn needs_redraw(&mut self) -> bool {
        for b in &mut self.bars {
            if b.needs_redraw() {
                return true;
            }
        }
        for l in &mut self.layouts {
            if l.needs_redraw() {
                return true;
            }
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
        if self.dim != dim {
            self.dim = dim;

            let x = self.margin.x;
            let mut y = self.margin.y;
            let width = dim.width;

            let sections = self.bars.len() as i32;
            let available_height = dim.height - sections * 22;

            let height_per_section = available_height / sections;

            for index in 0..sections {
                let i = index as usize;

                self.bars[i].set_dim(TheDim::new(dim.x + x, dim.y + y, width, 22));
                self.bars[i].dim_mut().set_buffer_offset(self.dim.buffer_x, self.dim.buffer_y + y);

                y += self.bars[i].dim().height;

                let mut dim = TheDim::new(dim.x + x, dim.y + y, width, height_per_section);
                dim.buffer_x = self.dim.buffer_x;
                dim.buffer_y = self.dim.buffer_y + y;
                self.layouts[i].set_dim(dim, ctx);

                y += height_per_section;
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
        if !self.dim().is_valid() {
            return;
        }

        for b in &mut self.bars {
            b.draw(buffer, style, ctx);
        }

        for l in &mut self.layouts {
            l.draw(buffer, style, ctx);
        }
    }
}

/// TheSnapperLayout specific functions.
pub trait TheSnapperLayoutTrait {
    /// Add a snapperbar / layout pair.
    fn add_pair(&mut self, snapperbar: Box<dyn TheWidget>, layout: Box<dyn TheLayout>);
}

impl TheSnapperLayoutTrait for TheSnapperLayout {
    fn add_pair(&mut self, snapperbar: Box<dyn TheWidget>, layout: Box<dyn TheLayout>) {
        self.bars.push(snapperbar);
        self.layouts.push(layout);
    }
}