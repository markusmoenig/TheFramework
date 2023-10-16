use crate::prelude::*;

pub struct TheSwitchbar {
    widget_id: TheWidgetId,

    dim: TheDim,
    text: String,
    is_dirty: bool,
}

impl TheWidget for TheSwitchbar {
    fn new(name: String) -> Self
    where
        Self: Sized,
    {
        Self {
            widget_id: TheWidgetId::new(name),

            dim: TheDim::zero(),
            text: "".to_string(),
            is_dirty: false,
        }
    }

    fn id(&self) -> &TheWidgetId {
        &self.widget_id
    }

    fn on_event(&mut self, event: &TheEvent, ctx: &mut TheContext) {
        /*
        println!("event ({}): {:?}", self.widget_id.name, event);
        match event {
            TheEvent::MouseDown(coord) => {
                ctx.ui.set_focus(self.id());
            }
            _ => {}
        }*/
    }

    fn dim(&self) -> &TheDim {
        &self.dim
    }

    fn dim_mut(&mut self) -> &mut TheDim {
        &mut self.dim
    }

    fn set_dim(&mut self, dim: TheDim) {
        if self.dim != dim {
            self.dim = dim;
            self.is_dirty = true;
        }
    }

    fn needs_redraw(&mut self) -> bool {
        self.is_dirty
    }

    fn draw(
        &mut self,
        buffer: &mut TheRGBABuffer,
        style: &mut Box<dyn TheStyle>,
        ctx: &mut TheContext,
    ) {
        let stride = buffer.stride();

        let utuple = self.dim.to_buffer_utuple();

        ctx.draw
            .rect_outline(buffer.pixels_mut(), &utuple, stride, style.theme().color(SwitchbarBorder));

        if let Some(icon) = ctx.ui.icon("dark_switchbar") {
            for x in 1..utuple.2 - 1 {
                let r = (utuple.0 + x, utuple.1, 1, icon.2 as usize);
                ctx.draw
                    .copy_slice_3(buffer.pixels_mut(), &icon.0, &r, stride);
            }
        }

        if let Some(icon) = ctx.ui.icon("switchbar_icon") {
            let r = (utuple.0 + 6, utuple.1 + 6, icon.1 as usize, icon.2 as usize);
            ctx.draw
                .blend_slice(buffer.pixels_mut(), &icon.0, &r, stride);
        }

        let mut shrinker = TheDimShrinker::zero();
        shrinker.shrink_by(30, 1, 0, 0);

        if let Some(font) = &ctx.ui.font {
            ctx.draw.text_rect_blend(
                buffer.pixels_mut(),
                &self.dim.to_buffer_shrunk_utuple(&shrinker),
                stride,
                font,
                15.0,
                &self.id().name,
                &WHITE,
                TheHorizontalAlign::Left,
                TheVerticalAlign::Center
            );
        }
    }
}

pub trait TheSectionHeaderTrait {
    fn set_text(&mut self, text: String);
}

impl TheSectionHeaderTrait for TheSwitchbar {
    fn set_text(&mut self, text: String) {
        self.text = text;
    }
}
