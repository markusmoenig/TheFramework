use crate::prelude::*;

pub struct TheColorButton {
    name: String,
    id: Uuid,

    dim: TheDim,

    color: RGBA
}

impl TheWidget for TheColorButton {
    fn new(name: String) -> Self
    where
        Self: Sized,
    {
        Self {
            name,
            id: Uuid::new_v4(),
            dim: TheDim::zero(),
            color : WHITE
        }
    }

    fn name(&self) -> &String { &self.name }
    fn id(&self) -> Uuid { self.id }

    fn on_event(&mut self, event: &TheEvent, _ctx: &mut TheContext) {
        //println!("event ({}): {:?}", self.name, event);
        match event {
            TheEvent::MouseDown(coord) => {

            },
            _ => {}
        }
    }

     fn dim(&self) -> &TheDim {
        &self.dim
     }


    /// Set the dimension of the widget
     fn set_dim(&mut self, dim: TheDim) {
        self.dim = dim;
    }

    fn draw(&mut self, buffer: &mut TheRGBABuffer, style: &mut Box<dyn TheStyle>, ctx: &mut TheContext) {
        let stride = buffer.stride();
        let mut shrinker = TheDimShrinker::zero();

        style.draw_widget_border(buffer, &self.dim, &mut shrinker, ctx);

        ctx.draw.rect(
            buffer.pixels_mut(),
            &self.dim.to_shrunk_utuple(&shrinker),
            stride,
            &self.color,
        );

        if let Some(font) = &ctx.ui.font {
            ctx.draw.text_rect_blend(buffer.pixels_mut(), &self.dim.to_shrunk_utuple(&shrinker), stride, font, 20.0, &self.name, &BLACK, crate::thedraw2d::TheTextAlignment::Center);
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
