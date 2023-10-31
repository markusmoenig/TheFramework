use crate::prelude::*;

pub struct TheListLayout {
    id: TheId,
    limiter: TheSizeLimiter,

    dim: TheDim,

    widgets: Vec<Box<dyn TheWidget>>,

    margin: Vec4i,

    background: Option<TheThemeColors>,
}

impl TheLayout for TheListLayout {
    fn new(name: String) -> Self
    where
        Self: Sized,
    {
        Self {
            id: TheId::new(name),
            limiter: TheSizeLimiter::new(),

            dim: TheDim::zero(),

            widgets: vec![],

            margin: vec4i(0, 0, 0, 0),

            background: Some(TextLayoutBackground),
        }
    }

    fn id(&self) -> &TheId {
        &self.id
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
        let widgets = self.widgets();
        widgets.iter_mut().find(|w| w.dim().contains(coord))
    }

    fn get_widget(
        &mut self,
        name: Option<&String>,
        uuid: Option<&Uuid>,
    ) -> Option<&mut Box<dyn TheWidget>> {
        self.widgets.iter_mut().find(|w| w.id().matches(name, uuid))
    }

    fn needs_redraw(&mut self) -> bool {
        for i in 0..self.widgets.len() {
            if self.widgets[i].needs_redraw() {
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

    fn set_dim(&mut self, dim: TheDim, _ctx: &mut TheContext) {
        if self.dim != dim {
            self.dim = dim;

            let x = 0;
            let mut y = 0;
            let width = dim.width;

            let items = self.widgets.len() as i32;
            // let mut total_height = items * 17;
            // if items > 0 {
            //     total_height += (items - 1) * 3;
            // }

            for index in 0..items {
                let i = index as usize;

                self.widgets[i].set_dim(TheDim::new(dim.x + x, dim.y + y, width, 17));
                self.widgets[i]
                    .dim_mut()
                    .set_buffer_offset(self.dim.buffer_x, self.dim.buffer_y + y);

                y += 17 + 3;
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

        let stride = buffer.stride();
        let utuple: (usize, usize, usize, usize) = self.dim.to_buffer_utuple();

        ctx.draw.rect(
            buffer.pixels_mut(),
            &utuple,
            stride,
            style.theme().color(ListLayoutBackground),
        );

        let items = self.widgets.len();
        for i in 0..items {
            self.widgets[i].draw(buffer, style, ctx);
        }
    }

    /// Convert to the list layout trait
    fn as_list_layout(&mut self) -> Option<&mut dyn TheListLayoutTrait> {
        Some(self)
    }
}

/// TheListLayout specific functions.
pub trait TheListLayoutTrait {
    /// Add an item
    fn add_item(&mut self, item: TheListItem);
    /// A new item was selected, manage the selection states
    fn new_item_selected(&mut self, item: TheId);
}

impl TheListLayoutTrait for TheListLayout {

    fn add_item(&mut self, mut item: TheListItem) {
        item.set_associated_layout(self.id().clone());
        self.widgets.push(Box::new(item));
    }
    fn new_item_selected(&mut self, item: TheId) {
        for w in &mut self.widgets {
            if !w.id().equals(&Some(item.clone())) {
                w.set_state(TheWidgetState::None);
            }
        }
    }
}
