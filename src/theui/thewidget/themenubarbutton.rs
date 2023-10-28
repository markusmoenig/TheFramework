use crate::prelude::*;

pub struct TheMenubarButton {
    widget_id: TheId,
    limiter: TheSizeLimiter,
    state: TheWidgetState,

    icon_name: String,
    icon_offset: Vec2i,

    dim: TheDim,
    is_dirty: bool,
}

impl TheWidget for TheMenubarButton {
    fn new(name: String) -> Self
    where
        Self: Sized,
    {
        let mut limiter = TheSizeLimiter::new();
        limiter.set_max_size(vec2i(35, 35));

        Self {
            widget_id: TheId::new(name),
            limiter,
            state: TheWidgetState::None,

            icon_name: "".to_string(),
            icon_offset: vec2i(0,0),

            dim: TheDim::zero(),
            is_dirty: false,
        }
    }

    fn id(&self) -> &TheId {
        &self.widget_id
    }

    #[allow(clippy::single_match)]
    fn on_event(&mut self, event: &TheEvent, ctx: &mut TheContext) -> bool {
        let mut redraw = false;
        // println!("event ({}): {:?}", self.widget_id.name, event);
        match event {
            TheEvent::MouseDown(_coord) => {
                if self.state != TheWidgetState::Clicked {
                    self.state = TheWidgetState::Clicked;
                    ctx.ui.set_focus(self.id());
                    ctx.ui.send_widget_state_changed(self.id(), self.state);
                }
                self.is_dirty = true;
                redraw = true;
            },
            TheEvent::Hover(_coord) => {
                if self.state != TheWidgetState::Clicked && !self.id().equals(&ctx.ui.hover) {
                    self.is_dirty = true;
                    ctx.ui.set_hover(self.id());
                    redraw = true;
                }
            },
            TheEvent::MouseUp(_coord) => {
                if self.state == TheWidgetState::Clicked {
                    self.state = TheWidgetState::None;
                    ctx.ui.clear_focus();
                }
                self.is_dirty = true;
                redraw = true;
            },
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

    fn draw(
        &mut self,
        buffer: &mut TheRGBABuffer,
        style: &mut Box<dyn TheStyle>,
        ctx: &mut TheContext,
    ) {
        let stride = buffer.stride();
        let mut shrinker = TheDimShrinker::zero();

        if !self.dim().is_valid() {
            return;
        }

        if self.state != TheWidgetState::None || self.id().equals(&ctx.ui.hover) {

            if self.state == TheWidgetState::Clicked {

                ctx.draw.rect_outline_border(
                    buffer.pixels_mut(),
                    &self.dim.to_buffer_shrunk_utuple(&shrinker),
                    stride,
                    style.theme().color(MenubarButtonClickedBorder),
                    1
                );

                shrinker.shrink(1);

                ctx.draw.rect(
                    buffer.pixels_mut(),
                    &self.dim.to_buffer_shrunk_utuple(&shrinker),
                    stride,
                    style.theme().color(MenubarButtonClicked),
                );
            } else if self.id().equals(&ctx.ui.hover) {

                ctx.draw.rect_outline_border(
                    buffer.pixels_mut(),
                    &self.dim.to_buffer_shrunk_utuple(&shrinker),
                    stride,
                    style.theme().color(MenubarButtonHoverBorder),
                    1
                );

                shrinker.shrink(1);

                ctx.draw.rect(
                    buffer.pixels_mut(),
                    &self.dim.to_buffer_shrunk_utuple(&shrinker),
                    stride,
                    style.theme().color(MenubarButtonHoverBorder),
                );
            }
        }

        if let Some(icon) = ctx.ui.icon(&self.icon_name) {
            let utuple = self.dim.to_buffer_shrunk_utuple(&shrinker);
            let r = (
                ((utuple.0 + (utuple.2 - icon.dim().width as usize) / 2) as i32 + self.icon_offset.x) as usize,
                ((utuple.1 + (utuple.3 - icon.dim().height as usize) / 2) as i32 + self.icon_offset.y) as usize,
                icon.dim().width as usize,
                icon.dim().height as usize,
            );
            ctx.draw
                .blend_slice(buffer.pixels_mut(), icon.pixels(), &r, stride);
        }

        self.is_dirty = false;
    }
}

pub trait TheMenubarButtonTrait {
    fn set_icon_name(&mut self, text: String);
    fn set_icon_offset(&mut self, offset: Vec2i);
}

impl TheMenubarButtonTrait for TheMenubarButton {
    fn set_icon_name(&mut self, text: String) {
        self.icon_name = text;
    }
    fn set_icon_offset(&mut self, offset: Vec2i) {
        self.icon_offset = offset;
    }
}