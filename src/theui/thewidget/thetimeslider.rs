use crate::prelude::*;

pub struct TheTimeSlider {
    id: TheId,
    limiter: TheSizeLimiter,
    state: TheWidgetState,

    value: TheValue,
    original: TheValue,

    default_value: TheValue,

    status: Option<String>,

    dim: TheDim,
    is_dirty: bool,

    continuous: bool,
}

impl TheWidget for TheTimeSlider {
    fn new(id: TheId) -> Self
    where
        Self: Sized,
    {
        let mut limiter = TheSizeLimiter::new();
        limiter.set_max_size(vec2i(300, 20));

        Self {
            id,
            limiter,

            state: TheWidgetState::None,

            value: TheValue::Time(TheTime::default()),
            original: TheValue::Float(0.0),

            default_value: TheValue::Float(1.0),

            status: None,

            dim: TheDim::zero(),
            is_dirty: false,

            continuous: false,
        }
    }

    fn id(&self) -> &TheId {
        &self.id
    }

    fn status_text(&self) -> Option<String> {
        self.status.clone()
    }

    fn set_status_text(&mut self, text: &str) {
        self.status = Some(text.to_string());
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
            self.value = value.clone();
            self.default_value = value;
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
                }

                ctx.ui.set_focus(self.id());

                self.value = TheValue::Time(TheTime::from_widget_offset(coord.x as u32, self.dim.width as u32));

                ctx.ui
                    .send_widget_value_changed(self.id(), self.value.clone());
                redraw = true;
            }
            TheEvent::MouseDragged(coord) => {

                let mut offset = coord.x;
                if offset < 0 {
                    offset = 0;
                }

                if offset > self.dim.width {
                    offset = self.dim.width ;
                }

                self.value = TheValue::Time(TheTime::from_widget_offset(offset as u32, self.dim.width as u32));

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
                }

                if self.value != self.original {
                    ctx.ui
                        .send_widget_value_changed(self.id(), self.value.clone());
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

        let r = self.dim.to_buffer_utuple();

        ctx.draw.rounded_rect(
            buffer.pixels_mut(),
            &r,
            stride,
            style.theme().color(TimeSliderBackground),
            &(5.0, 5.0, 5.0, 5.0)
        );

        let text_space = r.2 / 24;
        let mut x = r.0;

        for i in 0..=24 {
            if i > 0 {
                let r = (x, r.1 + 1, 1, 2);
                ctx.draw.rect(
                    buffer.pixels_mut(),
                    &r,
                    stride,
                    style.theme().color(TimeSliderText),
                );
            }
            x += text_space;
        }

        let text_space = r.2 / 12;
        x = r.0;

        for i in 0..12 {
            if i > 0 {
                let r = (x - text_space / 2, r.1, text_space, r.3 - 1);
                if let Some(font) = &ctx.ui.font {
                    ctx.draw.text_rect_blend(
                        buffer.pixels_mut(),
                        &r,
                        stride,
                        font,
                        11.0,
                        &(i * 2).to_string(),
                        style.theme().color(TimeSliderText),
                        TheHorizontalAlign::Center,
                        TheVerticalAlign::Bottom,
                    );
                }
            }
            x += text_space;
        }

        if let TheValue::Time(time) = &self.value {
            let offset = time.to_widget_offset(self.dim.width as u32) as usize;

            let r = (r.0 + offset, r.1, 2, r.3);
            ctx.draw.rect(
                buffer.pixels_mut(),
                &r,
                stride,
                style.theme().color(TimeSliderMarker),
            );
        }

        self.is_dirty = false;
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub trait TheTimeSliderTrait: TheWidget {
    fn set_continuous(&mut self, continuous: bool);
}

impl TheTimeSliderTrait for TheTimeSlider {
    fn set_continuous(&mut self, continuous: bool) {
        self.continuous = continuous;
    }
}
