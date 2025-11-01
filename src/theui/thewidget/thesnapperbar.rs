use crate::prelude::*;

#[derive(Default)]
pub struct TheSnapperbar {
    id: TheId,
    limiter: TheSizeLimiter,

    state: TheWidgetState,
    open: bool,
    collapse_uuid: Option<Uuid>,

    dim: TheDim,
    text: String,
    is_dirty: bool,
}

impl TheWidget for TheSnapperbar {
    fn new(id: TheId) -> Self
    where
        Self: Sized,
    {
        let mut limiter = TheSizeLimiter::new();
        limiter.set_max_height(22);

        Self {
            id,
            limiter,

            state: TheWidgetState::None,
            open: true,
            collapse_uuid: None,

            dim: TheDim::zero(),
            text: "".to_string(),
            is_dirty: false,
        }
    }

    fn id(&self) -> &TheId {
        &self.id
    }

    // fn on_event(&mut self, event: &TheEvent, ctx: &mut TheContext) -> bool {
    //     false
    // }

    fn dim(&self) -> &TheDim {
        &self.dim
    }

    fn dim_mut(&mut self) -> &mut TheDim {
        &mut self.dim
    }

    fn set_dim(&mut self, dim: TheDim, _ctx: &mut TheContext) {
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

    fn is_open(&self) -> bool {
        self.open
    }

    fn needs_redraw(&mut self) -> bool {
        self.is_dirty
    }

    fn set_needs_redraw(&mut self, redraw: bool) {
        self.is_dirty = redraw;
    }

    fn on_event(&mut self, event: &TheEvent, ctx: &mut TheContext) -> bool {
        let mut redraw = false;
        // println!("event ({}): {:?}", self.widget_id.name, event);
        match event {
            TheEvent::MouseDown(_coord) => {
                self.is_dirty = true;
                if self.state != TheWidgetState::Clicked {
                    self.state = TheWidgetState::Clicked;
                    ctx.ui.send_widget_state_changed(self.id(), self.state);
                    ctx.ui.set_focus(self.id());
                }
                redraw = true;
            }
            TheEvent::MouseUp(_coord) => {
                self.is_dirty = true;
                if self.state == TheWidgetState::Clicked {
                    self.state = TheWidgetState::None;
                    ctx.ui.send_widget_state_changed(self.id(), self.state);
                    self.open = !self.open;
                    ctx.ui.redraw_all = true;
                    ctx.ui.relayout = true;
                }
                redraw = true;
            }
            TheEvent::Hover(_coord) => {
                if self.state != TheWidgetState::Clicked && !self.id().equals(&ctx.ui.hover) {
                    self.is_dirty = true;
                    ctx.ui.set_hover(self.id());
                    redraw = true;
                }
            }
            _ => {}
        }
        redraw
    }

    fn draw(
        &mut self,
        buffer: &mut TheRGBABuffer,
        _style: &mut Box<dyn TheStyle>,
        ctx: &mut TheContext,
    ) {
        if !self.dim().is_valid() {
            return;
        }

        let stride = buffer.stride();
        let utuple: (usize, usize, usize, usize) = self.dim.to_buffer_utuple();

        let mut icon_state = if self.state == TheWidgetState::Clicked {
            "clicked".to_string()
        } else {
            "normal".to_string()
        };

        if self.state != TheWidgetState::Selected && self.id().equals(&ctx.ui.hover) {
            icon_state = "hover".to_string()
        }

        if let Some(icon) = ctx
            .ui
            .icon(format!("dark_snapperbar_{}_front", icon_state).as_str())
        {
            let r = (utuple.0, utuple.1 + 1, 1, icon.dim().height as usize);
            ctx.draw
                .copy_slice(buffer.pixels_mut(), icon.pixels(), &r, stride);

            let r = (
                utuple.0 + utuple.2 - 1,
                utuple.1 + 1,
                1,
                icon.dim().height as usize,
            );
            ctx.draw
                .copy_slice(buffer.pixels_mut(), icon.pixels(), &r, stride);
        }

        if let Some(icon) = ctx
            .ui
            .icon(format!("dark_snapperbar_{}_middle", icon_state).as_str())
        {
            for x in 1..utuple.2 - 1 {
                let r = (utuple.0 + x, utuple.1, 1, icon.dim().height as usize);
                ctx.draw
                    .copy_slice(buffer.pixels_mut(), icon.pixels(), &r, stride);
            }
        }

        if self.open {
            if let Some(icon) = ctx.ui.icon("dark_snapperbar_open") {
                let r = (
                    utuple.0 + 6,
                    utuple.1 + 9,
                    icon.dim().width as usize,
                    icon.dim().height as usize,
                );
                ctx.draw
                    .blend_slice(buffer.pixels_mut(), icon.pixels(), &r, stride);
            }
        } else if let Some(icon) = ctx.ui.icon("dark_snapperbar_closed") {
            let r = (
                utuple.0 + 9,
                utuple.1 + 6,
                icon.dim().width as usize,
                icon.dim().height as usize,
            );
            ctx.draw
                .blend_slice(buffer.pixels_mut(), icon.pixels(), &r, stride);
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
                TheVerticalAlign::Center,
            );
        }

        self.is_dirty = false;
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub trait TheSnapperbarTrait {
    fn set_text(&mut self, text: String);
    fn set_canvas_collapse_uuid(&mut self, collapse: Uuid);
    fn is_open(&self) -> bool;
}

impl TheSnapperbarTrait for TheSnapperbar {
    fn set_text(&mut self, text: String) {
        self.text = text;
        self.is_dirty = true;
    }
    fn set_canvas_collapse_uuid(&mut self, collapse: Uuid) {
        self.collapse_uuid = Some(collapse);
    }
    fn is_open(&self) -> bool {
        self.open
    }
}
