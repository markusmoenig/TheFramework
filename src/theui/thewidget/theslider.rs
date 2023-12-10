use crate::prelude::*;

pub struct TheSlider {
    id: TheId,
    limiter: TheSizeLimiter,
    state: TheWidgetState,

    value: TheValue,
    original: TheValue,

    text_width: i32,

    dim: TheDim,
    is_dirty: bool,

    range: TheValue,
    continuous: bool,
}

impl TheWidget for TheSlider {
    fn new(id: TheId) -> Self
    where
        Self: Sized,
    {
        let mut limiter = TheSizeLimiter::new();
        limiter.set_max_height(13);

        Self {
            id,
            limiter,

            state: TheWidgetState::None,

            value: TheValue::Float(0.0),
            original: TheValue::Float(0.0),

            text_width: 50,

            dim: TheDim::zero(),
            is_dirty: false,

            range: TheValue::RangeF32(0.0..=1.0),
            continuous: false,
        }
    }

    fn id(&self) -> &TheId {
        &self.id
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

    fn value(&self) -> TheValue {
        self.value.clone()
    }

    fn set_value(&mut self, value: TheValue) {
        if value != self.value {
            self.value = value;
            self.is_dirty = true;
        }
    }

    #[allow(clippy::single_match)]
    fn on_event(&mut self, event: &TheEvent, ctx: &mut TheContext) -> bool {
        let mut redraw = false;
        //println!("event ({}): {:?}", self.widget_id.name, event);
        match event {
            TheEvent::MouseDown(coord) => {
                self.is_dirty = true;
                if self.state != TheWidgetState::Selected {
                    self.state = TheWidgetState::Selected;
                    ctx.ui.send_widget_state_changed(self.id(), self.state);
                    ctx.ui.set_focus(self.id());
                    ctx.ui.set_overlay(self.id());
                }

                if let Some(range_f32) = self.range.to_range_f32() {
                    let d = abs(range_f32.end() - range_f32.start())
                        * (coord.x as f32 / (self.dim.width - self.text_width) as f32)
                            .clamp(0.0, 1.0);
                    let v = *range_f32.start() + d;
                    self.original = self.value.clone();
                    self.value = TheValue::Float(v);
                } else if let Some(range_i32) = self.range.to_range_i32() {
                    let range_diff = range_i32.end() - range_i32.start();
                    let d = (coord.x * range_diff) / (self.dim.width - self.text_width);
                    let v = (*range_i32.start() + d).clamp(*range_i32.start(), *range_i32.end());
                    self.original = self.value.clone();
                    self.value = TheValue::Int(v);
                }
                redraw = true;
            }
            TheEvent::MouseDragged(coord) => {
                if let Some(range_f32) = self.range.to_range_f32() {
                    let d = abs(range_f32.end() - range_f32.start())
                        * (coord.x as f32 / (self.dim.width - self.text_width) as f32)
                            .clamp(0.0, 1.0);
                    let v = *range_f32.start() + d;
                    self.original = self.value.clone();
                    self.value = TheValue::Float(v);
                } else if let Some(range_i32) = self.range.to_range_i32() {
                    let range_diff = range_i32.end() - range_i32.start();
                    let d = (coord.x * range_diff) / (self.dim.width - self.text_width);
                    let v = (*range_i32.start() + d).clamp(*range_i32.start(), *range_i32.end());
                    self.original = self.value.clone();
                    self.value = TheValue::Int(v);
                }
                if self.continuous {
                    ctx.ui
                        .send_widget_value_changed(self.id(), self.value.clone());
                }
                self.is_dirty = true;
                redraw = true;
            }
            TheEvent::MouseUp(_coord) => {
                self.is_dirty = true;
                if self.state == TheWidgetState::Selected {
                    self.state = TheWidgetState::None;

                    if self.value != self.original {
                        ctx.ui
                            .send_widget_value_changed(self.id(), self.value.clone());
                    }
                }
                redraw = true;
            }
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

    fn draw(
        &mut self,
        buffer: &mut TheRGBABuffer,
        style: &mut Box<dyn TheStyle>,
        ctx: &mut TheContext,
    ) {
        if !self.dim().is_valid() {
            return;
        }

        let stride = buffer.stride();
        let mut shrinker = TheDimShrinker::zero();

        shrinker.shrink_by(1, 5, 1, 0);

        let mut r = self.dim.to_buffer_shrunk_utuple(&shrinker);
        r.2 -= self.text_width as usize;
        r.3 = 1;

        ctx.draw.rect(
            buffer.pixels_mut(),
            &r,
            stride,
            style.theme().color(SliderSmallColor1),
        );

        shrinker.reset();
        shrinker.shrink_by(0, 6, 0, 0);
        r = self.dim.to_buffer_shrunk_utuple(&shrinker);
        r.2 -= self.text_width as usize;
        r.3 = 1;

        ctx.draw.rect(
            buffer.pixels_mut(),
            &r,
            stride,
            style.theme().color(SliderSmallColor4),
        );

        shrinker.shrink_by(0, 1, 0, 0);
        r = self.dim.to_buffer_shrunk_utuple(&shrinker);
        r.2 -= self.text_width as usize;
        r.3 = 1;

        ctx.draw.rect(
            buffer.pixels_mut(),
            &r,
            stride,
            style.theme().color(SliderSmallColor3),
        );

        shrinker.shrink_by(1, 1, 1, 0);
        r = self.dim.to_buffer_shrunk_utuple(&shrinker);
        r.2 -= self.text_width as usize;
        r.3 = 1;

        ctx.draw.rect(
            buffer.pixels_mut(),
            &r,
            stride,
            style.theme().color(SliderSmallColor3),
        );

        shrinker.reset();
        shrinker.shrink_by(1, 6, 1, 0);
        r = self.dim.to_buffer_shrunk_utuple(&shrinker);
        r.2 -= self.text_width as usize;
        r.3 = 2;

        ctx.draw.rect(
            buffer.pixels_mut(),
            &r,
            stride,
            style.theme().color(SliderSmallColor2),
        );

        let mut icon_name = if self.state == TheWidgetState::Selected {
            "dark_slider_small_selected".to_string()
        } else {
            "dark_slider_small_normal".to_string()
        };

        if self.state != TheWidgetState::Selected && self.id().equals(&ctx.ui.hover) {
            icon_name = "dark_slider_small_selected".to_string()
        }
        if self.state != TheWidgetState::Selected && self.id().equals(&ctx.ui.focus) {
            icon_name = "dark_slider_small_selected".to_string()
        }

        let mut pos = 0;
        let mut text = "".to_string();

        if let Some(range_f32) = self.range.to_range_f32() {
            if let Some(value) = self.value.to_f32() {
                let normalized =
                    (value - range_f32.start()) / (range_f32.end() - range_f32.start());
                pos = (normalized * (self.dim.width - self.text_width) as f32) as usize;
                text = format!("{:.2}", value);
            }
        } else if let Some(range_i32) = self.range.to_range_i32() {
            if let Some(value) = self.value.to_i32() {
                let range_diff = range_i32.end() - range_i32.start();
                let normalized =
                    (value - range_i32.start()) * (self.dim.width - self.text_width) / range_diff;
                pos = normalized as usize;
                text = format!("{:.2}", value);
            }
        }

        if let Some(icon) = ctx.ui.icon(&icon_name) {
            let utuple = self.dim.to_buffer_utuple();
            let r = (
                utuple.0 + pos,
                utuple.1,
                icon.dim().width as usize,
                icon.dim().height as usize,
            );
            ctx.draw
                .blend_slice(buffer.pixels_mut(), icon.pixels(), &r, stride);
        }

        shrinker.reset();
        shrinker.shrink_by(self.dim.width - self.text_width + 10, 0, 0, 0);

        if let Some(font) = &ctx.ui.font {
            ctx.draw.text_rect_blend(
                buffer.pixels_mut(),
                &self.dim.to_buffer_shrunk_utuple(&shrinker),
                stride,
                font,
                13.0,
                &text,
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

pub trait TheSliderTrait: TheWidget {
    fn set_range(&mut self, range: TheValue);
    fn set_continuous(&mut self, continuous: bool);
}

impl TheSliderTrait for TheSlider {
    fn set_range(&mut self, range: TheValue) {
        if range != self.range {
            self.range = range;
            self.is_dirty = true;
        }
    }
    fn set_continuous(&mut self, continuous: bool) {
        self.continuous = continuous;
    }
}
