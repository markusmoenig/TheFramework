use crate::prelude::*;

pub struct TheSectionbarButton {
    widget_id: TheWidgetId,

    state: TheWidgetState,

    dim: TheDim,
    text: String,
    is_dirty: bool,
}

impl TheWidget for TheSectionbarButton {
    fn new(name: String) -> Self
    where
        Self: Sized,
    {
        Self {
            widget_id: TheWidgetId::new(name),

            state: TheWidgetState::None,

            dim: TheDim::zero(),
            text: String::new(),
            is_dirty: false,
        }
    }

    fn id(&self) -> &TheWidgetId {
        &self.widget_id
    }

    fn on_event(&mut self, event: &TheEvent, ctx: &mut TheContext) -> bool {
        let mut redraw = false;
        // println!("event ({}): {:?}", self.widget_id.name, event);
        match event {
            TheEvent::MouseDown(_coord) => {
                self.is_dirty = true;
                if self.state != TheWidgetState::Selected {
                    self.state = TheWidgetState::Selected;
                    ctx.ui.send_widget_state_changed(self.id(), self.state);
                }
                redraw = true;
            },
            TheEvent::Hover(_coord) => {
                if self.state != TheWidgetState::Selected && !self.id().equals(&ctx.ui.hover) {
                    self.is_dirty = true;
                    ctx.ui.set_hover(self.id());
                    redraw = true;
                }
            }
            _ => {}
        }
        redraw
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

    fn state(&self) -> TheWidgetState { self.state }

    fn set_state(&mut self, state: TheWidgetState) {
        self.state = state;
        self.is_dirty = true;
    }

    fn supports_hover(&mut self) -> bool {
        true
    }

    fn needs_redraw(&mut self) -> bool {
        self.is_dirty
    }

    fn set_needs_redraw(&mut self, redraw: bool) {
        self.is_dirty = redraw;
    }

    fn draw(
        &mut self,
        buffer: &mut TheRGBABuffer,
        style: &mut Box<dyn TheStyle>,
        ctx: &mut TheContext,
    ) {
        let stride = buffer.stride();
        let shrinker = TheDimShrinker::zero();

        let utuple: (usize, usize, usize, usize) = self.dim.to_buffer_utuple();

        let mut icon_name =  if self.state == TheWidgetState::Selected {
            "dark_sectionbarbutton_selected".to_string()
        } else {
            "dark_sectionbarbutton_normal".to_string()
        };

        if self.state != TheWidgetState::Selected && self.id().equals(&ctx.ui.hover) {
            icon_name = "dark_sectionbarbutton_hover".to_string()
        }

        let text_color =  if self.state == TheWidgetState::Selected {
            style.theme().color(SectionbarSelectedTextColor)
        } else {
            style.theme().color(SectionbarNormalTextColor)
        };

        if let Some(icon) = ctx.ui.icon(&icon_name) {
            let r = (utuple.0, utuple.1, icon.1 as usize, icon.2 as usize);
            ctx.draw
                .blend_slice(buffer.pixels_mut(), &icon.0, &r, stride);
        }

        if let Some(font) = &ctx.ui.font {
            ctx.draw.text_rect_blend(
                buffer.pixels_mut(),
                &self.dim.to_buffer_shrunk_utuple(&shrinker),
                stride,
                font,
                15.0,
                &self.text,
                text_color,
                TheHorizontalAlign::Center,
                TheVerticalAlign::Center,
            );
        }

        self.is_dirty = false;
    }
}

pub trait TheSectionbarButtonTrait {
    fn set_text(&mut self, text: String);
}

impl TheSectionbarButtonTrait for TheSectionbarButton {
    fn set_text(&mut self, text: String) {
        self.text = text;
    }
}
