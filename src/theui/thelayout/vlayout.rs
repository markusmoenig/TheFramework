use crate::prelude::*;

pub struct TheVLayout {
    widget_id: TheWidgetId,
    limiter: TheSizeLimiter,

    dim: TheDim,

    widgets: Vec<Box<dyn TheWidget>>,

    margin: Vec4i,
    padding: i32,

    background: Option<TheThemeColors>,
}

impl TheLayout for TheVLayout {
    fn new(name: String) -> Self
    where
        Self: Sized,
    {
        Self {
            widget_id: TheWidgetId::new(name),
            limiter: TheSizeLimiter::new(),

            dim: TheDim::zero(),

            widgets: vec![],

            margin: vec4i(10, 10, 10, 10),
            padding: 5,

            background: Some(DefaultWidgetBackground),
        }
    }

    fn id(&self) -> &TheWidgetId {
        &self.widget_id
    }

    fn set_margin(&mut self, margin: Vec4i) {
        self.margin = margin;
    }

    fn set_padding(&mut self, padding: i32) {
        self.padding = padding;
    }

    fn set_background_color(&mut self, color: Option<TheThemeColors>) {
        self.background = color;
    }

    fn widgets(&mut self) -> &mut Vec<Box<dyn TheWidget>> {
        &mut self.widgets
    }

    fn add_widget(&mut self, widget: Box<dyn TheWidget>) {
        self.widgets.push(widget);
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

    // fn on_event(&mut self, event: &TheEvent, ctx: &mut TheContext) {
    //     println!("event ({}): {:?}", self.widget_id.name, event);
    //     match event {
    //         TheEvent::MouseDown(coord) => {
    //             ctx.ui.set_focus(self.id());
    //         }
    //         _ => {}
    //     }
    // }

    fn dim(&self) -> &TheDim {
        &self.dim
    }

    fn dim_mut(&mut self) -> &mut TheDim {
        &mut self.dim
    }

    fn set_dim(&mut self, dim: TheDim) {
        if self.dim != dim {
            self.dim = dim;

            let x = self.margin.x;
            let mut y = self.margin.y;

            for w in &mut self.widgets {
                let width = w.limiter().get_width(dim.width);
                let height = w.limiter().get_height(dim.height);

                // Limit to visible area
                if y + height > dim.height {
                    break;
                }

                w.set_dim(TheDim::new(dim.x + x, dim.y + y, width, height));
                w.dim_mut()
                    .set_buffer_offset(self.dim.buffer_x + x, self.dim.buffer_y + y);
                y += height + self.padding;
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
        if let Some(background) = self.background {
            let stride = buffer.stride();

            ctx.draw.rect(
                buffer.pixels_mut(),
                &self.dim.to_buffer_utuple(),
                stride,
                style.theme().color(background),
            );
        }

        for w in &mut self.widgets {
            w.draw(buffer, style, ctx);
        }
    }
}
