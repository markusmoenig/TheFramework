use crate::prelude::*;

pub struct TheVLayout {
    widget_id: TheWidgetId,

    dim: TheDim,

    widgets: Vec<Box<dyn TheWidget>>,
}

impl TheLayout for TheVLayout {
    fn new(name: String) -> Self
    where
        Self: Sized,
    {
        Self {
            widget_id: TheWidgetId::new(name),

            dim: TheDim::zero(),

            widgets: vec![],
        }
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

            let x = 10;
            let mut y = 10;

            for w in &mut self.widgets {
                w.set_dim(TheDim::new(dim.x + x, dim.y + y, 60, 60));
                w.dim_mut().set_buffer_offset(x, y);
                y += 40;
            }
        }
    }

    fn draw(
        &mut self,
        buffer: &mut TheRGBABuffer,
        style: &mut Box<dyn TheStyle>,
        ctx: &mut TheContext,
    ) {
        let stride = buffer.stride();

        ctx.draw.rect(
            buffer.pixels_mut(),
            &self.dim.to_buffer_utuple(),
            stride,
            style.theme().color(DefaultWidgetBackground),
        );

        let mut redraw = false;

        for w in &mut self.widgets {
            if w.needs_redraw() {
                redraw = true;
                break;
            }
        }

        //if redraw {
            for w in &mut self.widgets {
                w.draw(buffer, style, ctx);
            }
        //}
    }
}
