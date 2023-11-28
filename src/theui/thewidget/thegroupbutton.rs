use crate::prelude::*;

pub struct TheGroupButton {
    id: TheId,
    limiter: TheSizeLimiter,
    state: TheWidgetState,

    texts: Vec<String>,
    icons: Vec<Option<TheRGBABuffer>>,

    hover_index: Option<usize>,
    selected_index: Option<usize>,

    item_width: usize,

    dim: TheDim,

    is_disabled: bool,
    is_dirty: bool,
}

impl TheWidget for TheGroupButton {
    fn new(id: TheId) -> Self
    where
        Self: Sized,
    {
        let mut limiter = TheSizeLimiter::new();
        limiter.set_max_size(vec2i(60, 20));
        Self {
            id,
            limiter,
            state: TheWidgetState::None,

            texts: vec![],
            icons: vec![],

            hover_index: Some(0),
            selected_index: None,

            item_width: 60,

            dim: TheDim::zero(),

            is_disabled: false,
            is_dirty: false,
        }
    }

    fn id(&self) -> &TheId {
        &self.id
    }

    #[allow(clippy::single_match)]
    fn on_event(&mut self, event: &TheEvent, ctx: &mut TheContext) -> bool {
        let mut redraw = false;
        // println!("event ({}): {:?}", self.widget_id.name, event);
        match event {
            TheEvent::MouseDown(coord) => {
                if self.state != TheWidgetState::Selected {
                    self.state = TheWidgetState::Selected;
                    ctx.ui.send_widget_state_changed(self.id(), self.state);
                }
                if let Some(coord) = coord.to_vec2i() {
                    let index = coord.x as usize / (self.item_width + 1);
                    ctx.ui.send(TheEvent::IndexChanged(self.id.clone(), index));
                    self.selected_index = Some(index);
                }
                self.is_dirty = true;
                redraw = true;
            }
            TheEvent::Hover(coord) => {
                if !self.id().equals(&ctx.ui.hover) {
                    self.is_dirty = true;
                    ctx.ui.set_hover(self.id());
                    redraw = true;
                }
                if let Some(coord) = coord.to_vec2i() {
                    let index = coord.x as usize / (self.item_width + 1);
                    if Some(index) != self.hover_index && Some(index) != self.selected_index {
                        self.hover_index = Some(index);
                        redraw = true;
                        self.is_dirty = true;
                    }
                }
            }
            TheEvent::LostHover(_id) => {
                self.hover_index = None;
                redraw = true;
                self.is_dirty = true;
            }
            _ => {}
        }
        redraw
    }

    fn calculate_size(&mut self, _ctx: &mut TheContext) {
        let mut width = self.texts.len() * self.item_width;
        if !self.texts.is_empty() {
            width += self.texts.len() - 1;
        }
        self.limiter.set_max_width(width as i32);
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

    fn set_state(&mut self, state: TheWidgetState) {
        self.state = state;
        self.is_dirty = true;
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

    fn supports_hover(&mut self) -> bool {
        true
    }

    fn draw(
        &mut self,
        buffer: &mut TheRGBABuffer,
        style: &mut Box<dyn TheStyle>,
        ctx: &mut TheContext,
    ) {
        let stride: usize = buffer.stride();

        if !self.dim().is_valid() {
            return;
        }

        let ut = self.dim.to_buffer_utuple();

        //style.draw_widget_border(buffer, self, &mut shrinker, ctx);

        let total = self.texts.len() as i32;

        let mut x = 0;

        for (index, text) in self.texts.iter().enumerate() {

            let border;
            let bg;

            if self.selected_index == Some(index) {
                border = *style.theme().color(GroupButtonSelectedBorder);
                bg = *style.theme().color(GroupButtonSelectedBackground);
            } else if self.hover_index == Some(index) {
                border = *style.theme().color(GroupButtonHoverBorder);
                bg = *style.theme().color(GroupButtonHoverBackground);
            } else {
                border = *style.theme().color(GroupButtonNormalBorder);
                bg = *style.theme().color(GroupButtonNormalBackground);
            }

            if index == 0 {
                // First

                ctx.draw.rect_outline_border(
                    buffer.pixels_mut(),
                    &(ut.0 + x, ut.1, self.item_width, 20),
                    stride,
                    &border,
                    1,
                );

                ctx.draw.rect(
                    buffer.pixels_mut(),
                    &(ut.0 + x + self.item_width - 1, ut.1, 1, 20),
                    stride,
                    &border,
                );

                ctx.draw.rect(
                    buffer.pixels_mut(),
                    &(ut.0 + x + 1, ut.1 + 1, self.item_width - 1, 18),
                    stride,
                    &bg,
                );
            } else if index == total as usize - 1 {
                // Last

                ctx.draw.rect_outline_border(
                    buffer.pixels_mut(),
                    &(ut.0 + x, ut.1, self.item_width, 20),
                    stride,
                    &border,
                    1,
                );

                ctx.draw.rect(
                    buffer.pixels_mut(),
                    &(ut.0 + x, ut.1, self.item_width - 2, 20),
                    stride,
                    &border,
                );

                ctx.draw.rect(
                    buffer.pixels_mut(),
                    &(ut.0 + x, ut.1 + 1, self.item_width - 1, 18),
                    stride,
                    &bg,
                );
            } else {
                ctx.draw.rect_outline (
                    buffer.pixels_mut(),
                    &(ut.0 + x, ut.1, self.item_width, 20),
                    stride,
                    &border,
                );

                ctx.draw.rect(
                    buffer.pixels_mut(),
                    &(ut.0 + x, ut.1 + 1, self.item_width, 18),
                    stride,
                    &bg,
                );
            }

            if let Some(font) = &ctx.ui.font {
                ctx.draw.text_rect_blend(
                    buffer.pixels_mut(),
                    &(ut.0 + x + 1, ut.1 + 1, self.item_width - 2, 18),
                    stride,
                    font,
                    12.5,
                    text,
                    &WHITE,
                    TheHorizontalAlign::Center,
                    TheVerticalAlign::Center,
                );
            }

            x += self.item_width;
            if (index as i32) < total {
                x += 1;
            }
        }

        /*
        ctx.draw.rect_outline_border(
            buffer.pixels_mut(),
            &self.dim.to_buffer_shrunk_utuple(&shrinker),
            stride,
            &self.color,
            1,
        );

        if self.state == TheWidgetState::Selected {
            shrinker.shrink(1);
            ctx.draw.rect(
                buffer.pixels_mut(),
                &self.dim.to_buffer_shrunk_utuple(&shrinker),
                stride,
                &self.color,
            );
            ctx.draw.rect(
                buffer.pixels_mut(),
                &self.dim.to_buffer_shrunk_utuple(&shrinker),
                stride,
                &self.color,
            );
        }*/

        self.is_dirty = false;
    }

    fn as_group_button(&mut self) -> Option<&mut dyn TheGroupButtonTrait> {
        Some(self)
    }
}

pub trait TheGroupButtonTrait {
    fn add_text(&mut self, text: String);
}

impl TheGroupButtonTrait for TheGroupButton {
    fn add_text(&mut self, text: String) {
        self.texts.push(text);
        self.icons.push(None);
    }
}
