use crate::prelude::*;

pub struct TheSectionHeader {
    widget_id: TheWidgetId,
    widget_state: TheWidgetState,

    dim: TheDim,

    text: String,
}

impl TheWidget for TheSectionHeader {
    fn new(name: String) -> Self
    where
        Self: Sized,
    {
        Self {
            widget_id: TheWidgetId::new(name),
            widget_state: TheWidgetState::new(),

            dim: TheDim::zero(),
            text: "".to_string(),
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
    }

    fn draw(
        &mut self,
        buffer: &mut TheRGBABuffer,
        style: &mut Box<dyn TheStyle>,
        ctx: &mut TheContext,
    ) {
        let stride = buffer.stride();

        let utuple = self.dim.to_buffer_utuple();

        ctx.draw.rect_outline(
            buffer.pixels_mut(),
            &utuple,
            stride,
            &BLACK,
        );

        if let Some(icon) = ctx.ui.icon("dark_sectionheader") {
            for x in 1..utuple.2-1 {
                let r = (utuple.0 + x, utuple.1, 1, icon.2 as usize);
                ctx.draw.copy_slice_3(buffer.pixels_mut(), &icon.0, &r, stride);
            }
        }

        if let Some(icon) = ctx.ui.icon("caret-double-right-fill") {
            let r = (utuple.0 + 5, utuple.1 + 3, icon.1 as usize, icon.2 as usize);
            ctx.draw.copy_slice(buffer.pixels_mut(), &icon.0, &r, stride);
        }

        if let Some(font) = &ctx.ui.font {
            ctx.draw.text_rect_blend(
                buffer.pixels_mut(),
                &self.dim.to_buffer_utuple(),
                stride,
                font,
                16.0,
                &self.id().name,
                &WHITE,
                crate::thedraw2d::TheTextAlignment::Center,
            );
        }


        /*
        style.draw_widget_border(buffer, &self.dim, &mut shrinker, ctx);

        ctx.draw.rect(
            buffer.pixels_mut(),
            &self.dim.to_buffer_shrunk_utuple(&shrinker),
            stride,
            &self.color,
        );

        if let Some(font) = &ctx.ui.font {
            ctx.draw.text_rect_blend(
                buffer.pixels_mut(),
                &self.dim.to_buffer_shrunk_utuple(&shrinker),
                stride,
                font,
                20.0,
                &self.id().name,
                &BLACK,
                crate::thedraw2d::TheTextAlignment::Center,
            );
        }*/
    }
}

pub trait TheSectionHeaderTrait {
    fn set_text(&mut self, text: String);
}

impl TheSectionHeaderTrait for TheSectionHeader {
    fn set_text(&mut self, text: String) {
        self.text = text;
    }
}