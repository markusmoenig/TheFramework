use crate::prelude::*;

pub struct TheColorButton {
    widget_id: TheWidgetId,
    widget_state: TheWidgetState,

    dim: TheDim,

    color: RGBA,
}

impl TheWidget for TheColorButton {
    fn new(name: String) -> Self
    where
        Self: Sized,
    {
        Self {
            widget_id: TheWidgetId::new(name),
            widget_state: TheWidgetState::new(),

            dim: TheDim::zero(),
            color: WHITE,
        }
    }

    fn id(&self) -> &TheWidgetId {
        &self.widget_id
    }
    fn state(&self) -> &TheWidgetState {
        &self.widget_state
    }

    fn on_event(&mut self, event: &TheEvent, ctx: &mut TheContext) {
        println!("event ({}): {:?}", self.widget_id.name, event);
        match event {
            TheEvent::MouseDown(coord) => {
                ctx.ui.set_focus(self.id());
            }
            _ => {}
        }
    }

    fn dim(&self) -> &TheDim {
        &self.dim
    }

    fn dim_mut(&mut self) -> &mut TheDim {
        &mut self.dim
    }

    fn set_dim(&mut self, dim: TheDim) {
        self.dim = dim;
        println!("set_dim ({}): {:?}", self.widget_id.name, dim);
    }

    fn draw(
        &mut self,
        buffer: &mut TheRGBABuffer,
        style: &mut Box<dyn TheStyle>,
        ctx: &mut TheContext,
    ) {
        let stride = buffer.stride();
        let mut shrinker = TheDimShrinker::zero();

        style.draw_widget_border(buffer, &self.dim, &mut shrinker, ctx);

        println!(
            "draw ({}): {:?}",
            self.widget_id.name,
            self.dim.to_local_shrunk_utuple(&shrinker)
        );

        ctx.draw.rect(
            buffer.pixels_mut(),
            &self.dim.to_local_shrunk_utuple(&shrinker),
            stride,
            &self.color,
        );

        if let Some(font) = &ctx.ui.font {
            ctx.draw.text_rect_blend(
                buffer.pixels_mut(),
                &self.dim.to_local_shrunk_utuple(&shrinker),
                stride,
                font,
                20.0,
                &self.id().name,
                &BLACK,
                crate::thedraw2d::TheTextAlignment::Center,
            );
        }
    }
}

pub trait TheColorColorButtonTrait {
    fn set_color(&mut self, color: RGBA);
}

impl TheColorColorButtonTrait for TheColorButton {
    fn set_color(&mut self, color: RGBA) {
        self.color = color;
    }
}
