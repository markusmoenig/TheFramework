use crate::prelude::*;

pub struct TheTraybarButton {
    id: TheId,
    limiter: TheSizeLimiter,
    state: TheWidgetState,

    is_disabled: bool,

    icon_name: String,
    icon_offset: Vec2i,

    text: String,
    text_size: f32,

    dim: TheDim,
    is_dirty: bool,
}

impl TheWidget for TheTraybarButton {
    fn new(id: TheId) -> Self
    where
        Self: Sized,
    {
        let mut limiter = TheSizeLimiter::new();
        limiter.set_max_size(vec2i(20, 20));

        Self {
            id,
            limiter,
            state: TheWidgetState::None,

            icon_name: "".to_string(),
            icon_offset: vec2i(0, 0),

            text: "".to_string(),
            text_size: 13.0,

            dim: TheDim::zero(),
            is_dirty: false,
            is_disabled: false,
        }
    }

    fn id(&self) -> &TheId {
        &self.id
    }

    #[allow(clippy::single_match)]
    fn on_event(&mut self, event: &TheEvent, ctx: &mut TheContext) -> bool {
        if self.is_disabled {
            return false;
        }
        let mut redraw = false;
        //println!("event ({}): {:?}", self.id.name, event);
        match event {
            TheEvent::MouseDown(_coord) => {
                if self.state != TheWidgetState::Clicked {
                    //self.state = TheWidgetState::Clicked;
                    ctx.ui.set_focus(self.id());
                    ctx.ui.send_widget_state_changed(self.id(), self.state);
                }
                self.is_dirty = true;
                redraw = true;
            }
            TheEvent::Hover(_coord) => {
                if self.state != TheWidgetState::Clicked && !self.id().equals(&ctx.ui.hover) {
                    self.is_dirty = true;
                    ctx.ui.set_hover(self.id());
                    redraw = true;
                }
            }
            TheEvent::MouseUp(_coord) => {
                if self.state == TheWidgetState::Clicked {
                    self.state = TheWidgetState::None;
                    ctx.ui.clear_focus();
                }
                self.is_dirty = true;
                redraw = true;
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

    fn disabled(&self) -> bool {
        self.is_disabled
    }

    fn set_disabled(&mut self, disabled: bool) {
        if disabled != self.is_disabled {
            self.is_disabled = disabled;
            self.is_dirty = true;
            self.state = TheWidgetState::None;
        }
    }

    fn limiter(&self) -> &TheSizeLimiter {
        &self.limiter
    }

    fn limiter_mut(&mut self) -> &mut TheSizeLimiter {
        &mut self.limiter
    }

    fn needs_redraw(&mut self) -> bool {
        self.is_dirty
    }

    fn set_needs_redraw(&mut self, redraw: bool) {
        self.is_dirty = redraw;
    }

    fn state(&self) -> TheWidgetState {
        self.state
    }

    fn set_state(&mut self, state: TheWidgetState) {
        self.state = state;
        self.is_dirty = true;
    }

    fn supports_hover(&mut self) -> bool {
        true
    }

    fn calculate_size(&mut self, ctx: &mut TheContext) {
        if !self.text.is_empty() {
            if let Some(font) = &ctx.ui.font {
                let size = ctx.draw.get_text_size(font, self.text_size, &self.text);
                self.limiter_mut()
                    .set_max_width(ceil(size.0 as f32) as i32 + 15);
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
        let mut shrinker: TheDimShrinker = TheDimShrinker::zero();

        if !self.dim().is_valid() {
            return;
        }

        if self.is_disabled {
            ctx.draw.rect_outline_border(
                buffer.pixels_mut(),
                &self.dim.to_buffer_shrunk_utuple(&shrinker),
                stride,
                style.theme().color(TraybarButtonDisabledBorder),
                1,
            );

            shrinker.shrink(1);

            ctx.draw.rect(
                buffer.pixels_mut(),
                &self.dim.to_buffer_shrunk_utuple(&shrinker),
                stride,
                style.theme().color(TraybarButtonDisabledBackground),
            );
        }

        if !self.is_disabled
            && self.state == TheWidgetState::None
            && !self.id().equals(&ctx.ui.hover)
        {
            ctx.draw.rect_outline_border(
                buffer.pixels_mut(),
                &self.dim.to_buffer_shrunk_utuple(&shrinker),
                stride,
                style.theme().color(TraybarButtonNormalBorder),
                1,
            );

            shrinker.shrink(1);

            ctx.draw.rect(
                buffer.pixels_mut(),
                &self.dim.to_buffer_shrunk_utuple(&shrinker),
                stride,
                style.theme().color(TraybarButtonNormal),
            );
        }

        if !self.is_disabled && self.state != TheWidgetState::None
            || self.id().equals(&ctx.ui.hover)
        {
            if self.state == TheWidgetState::Clicked {
                ctx.draw.rect_outline_border(
                    buffer.pixels_mut(),
                    &self.dim.to_buffer_shrunk_utuple(&shrinker),
                    stride,
                    style.theme().color(TraybarButtonClickedBorder),
                    1,
                );

                shrinker.shrink(1);

                ctx.draw.rect(
                    buffer.pixels_mut(),
                    &self.dim.to_buffer_shrunk_utuple(&shrinker),
                    stride,
                    style.theme().color(TraybarButtonClicked),
                );
            } else if self.id().equals(&ctx.ui.hover) {
                ctx.draw.rect_outline_border(
                    buffer.pixels_mut(),
                    &self.dim.to_buffer_shrunk_utuple(&shrinker),
                    stride,
                    style.theme().color(TraybarButtonHover),
                    1,
                );

                shrinker.shrink(1);

                ctx.draw.rect(
                    buffer.pixels_mut(),
                    &self.dim.to_buffer_shrunk_utuple(&shrinker),
                    stride,
                    style.theme().color(TraybarButtonHoverBorder),
                );
            }
        }

        if let Some(icon) = ctx.ui.icon(&self.icon_name) {
            let utuple = self.dim.to_buffer_shrunk_utuple(&shrinker);
            let r = (
                ((utuple.0 + (utuple.2 - icon.dim().width as usize) / 2) as i32
                    + self.icon_offset.x) as usize,
                ((utuple.1 + (utuple.3 - icon.dim().height as usize) / 2) as i32
                    + self.icon_offset.y) as usize,
                icon.dim().width as usize,
                icon.dim().height as usize,
            );
            ctx.draw
                .blend_slice(buffer.pixels_mut(), icon.pixels(), &r, stride);
        }

        if !self.text.is_empty() {
            if let Some(font) = &ctx.ui.font {
                ctx.draw.text_rect_blend(
                    buffer.pixels_mut(),
                    &self.dim.to_buffer_shrunk_utuple(&shrinker),
                    stride,
                    font,
                    self.text_size,
                    &self.text,
                    &WHITE,
                    TheHorizontalAlign::Center,
                    TheVerticalAlign::Center,
                );
            }
        }

        self.is_dirty = false;
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub trait TheTraybarButtonTrait {
    fn set_icon_name(&mut self, text: String);
    fn set_icon_offset(&mut self, offset: Vec2i);
    fn set_text(&mut self, text: String);
}

impl TheTraybarButtonTrait for TheTraybarButton {
    fn set_icon_name(&mut self, text: String) {
        self.icon_name = text;
    }
    fn set_icon_offset(&mut self, offset: Vec2i) {
        self.icon_offset = offset;
    }
    fn set_text(&mut self, text: String) {
        self.text = text;
    }
}
