use crate::prelude::*;

pub struct TheListItem {
    id: TheId,
    limiter: TheSizeLimiter,

    state: TheWidgetState,

    text: String,
    sub_text: String,

    dim: TheDim,
    is_dirty: bool,

    mouse_down_pos: Vec2i,

    icon: Option<TheRGBABuffer>,

    layout_id: TheId,
}

impl TheWidget for TheListItem {
    fn new(id: TheId) -> Self
    where
        Self: Sized,
    {
        let mut limiter = TheSizeLimiter::new();
        limiter.set_max_height(17);

        Self {
            id,
            limiter,

            state: TheWidgetState::None,

            text: "".to_string(),
            sub_text: "".to_string(),

            dim: TheDim::zero(),
            is_dirty: true,

            mouse_down_pos: Vec2i::zero(),

            icon: None,

            layout_id: TheId::empty(),
        }
    }

    fn id(&self) -> &TheId {
        &self.id
    }

    fn on_event(&mut self, event: &TheEvent, ctx: &mut TheContext) -> bool {
        let mut redraw = false;
        // println!("event ({}): {:?}", self.widget_id.name, event);
        match event {
            TheEvent::Context(coord) => {
                ctx.ui
                    .send(TheEvent::ShowContextMenu(self.id().clone(), *coord));
            }
            TheEvent::MouseDown(_coord) => {
                if self.state != TheWidgetState::Selected || !self.id().equals(&ctx.ui.focus) {
                    self.is_dirty = true;
                    self.state = TheWidgetState::Selected;
                    ctx.ui.send_widget_state_changed(self.id(), self.state);
                    ctx.ui.send(TheEvent::NewListItemSelected(
                        self.id().clone(),
                        self.layout_id.clone(),
                    ));
                    redraw = true;
                }
                ctx.ui.set_focus(self.id());
            }
            TheEvent::MouseDragged(coord) => {
                if ctx.ui.drop.is_none()
                    && distance(Vec2f::from(self.mouse_down_pos), Vec2f::from(*coord)) >= 5.0
                {
                    ctx.ui.send(TheEvent::DragStarted(
                        self.id().clone(),
                        self.text.clone(),
                        *coord,
                    ));
                }
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
        TheValue::Text(self.text.clone())
    }

    fn set_value(&mut self, value: TheValue) {
        match value {
            TheValue::Empty => {
                self.text = "".to_string();
                self.is_dirty = true;
            }
            TheValue::Text(text) => {
                self.text = text.clone();
                self.is_dirty = true;
            }
            _ => {}
        }
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

        let mut color = if self.state == TheWidgetState::Selected {
            if !self.id().equals(&ctx.ui.focus) {
                style.theme().color(ListItemSelectedNoFocus)
            } else {
                style.theme().color(ListItemSelected)
            }
        } else {
            style.theme().color(ListItemNormal)
        };

        if self.state != TheWidgetState::Selected && self.id().equals(&ctx.ui.hover) {
            color = style.theme().color(ListItemHover);
        }

        let stride = buffer.stride();
        let mut shrinker = TheDimShrinker::zero();
        //shrinker.shrink_by(0, 0, 0, 0);

        ctx.draw.rect_outline_border(
            buffer.pixels_mut(),
            &self.dim.to_buffer_shrunk_utuple(&shrinker),
            stride,
            color,
            1,
        );

        shrinker.shrink(1);
        ctx.draw.rect(
            buffer.pixels_mut(),
            &self.dim.to_buffer_shrunk_utuple(&shrinker),
            stride,
            color,
        );

        if let Some(icon) = &self.icon {
            let ut = self.dim.to_buffer_shrunk_utuple(&shrinker);
            ctx.draw.rect_outline_border(
                buffer.pixels_mut(),
                &(ut.0 + 1, ut.1 + 1, 38, 38),
                stride,
                style.theme().color(ListItemIconBorder),
                1,
            );
            ctx.draw.copy_slice(
                buffer.pixels_mut(),
                icon.pixels(),
                &(ut.0 + 2, ut.1 + 2, 36, 36),
                stride,
            );

            if let Some(font) = &ctx.ui.font {
                ctx.draw.text_rect_blend(
                    buffer.pixels_mut(),
                    &(
                        ut.0 + 38 + 7 + 5,
                        ut.1 + 5,
                        (self.dim.width - 38 - 7 - 10) as usize,
                        13,
                    ),
                    stride,
                    font,
                    12.0,
                    &self.text,
                    style.theme().color(ListItemText),
                    TheHorizontalAlign::Left,
                    TheVerticalAlign::Center,
                );

                if !self.sub_text.is_empty() {
                    ctx.draw.text_rect_blend(
                        buffer.pixels_mut(),
                        &(
                            ut.0 + 38 + 7 + 5,
                            ut.1 + 22,
                            (self.dim.width - 38 - 7 - 10) as usize,
                            13,
                        ),
                        stride,
                        font,
                        12.0,
                        &self.sub_text,
                        style.theme().color(ListItemText),
                        TheHorizontalAlign::Left,
                        TheVerticalAlign::Center,
                    );
                }
            }
        } else {
            shrinker.shrink_by(9, 0, 0, 0);

            if let Some(font) = &ctx.ui.font {
                ctx.draw.text_rect_blend(
                    buffer.pixels_mut(),
                    &self.dim.to_buffer_shrunk_utuple(&shrinker),
                    stride,
                    font,
                    13.0,
                    &self.text,
                    style.theme().color(ListItemText),
                    TheHorizontalAlign::Left,
                    TheVerticalAlign::Center,
                );
            }
        }

        self.is_dirty = false;
    }

    fn as_list_item(&mut self) -> Option<&mut dyn TheListItemTrait> {
        Some(self)
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub trait TheListItemTrait {
    fn set_text(&mut self, text: String);
    fn set_sub_text(&mut self, sub_text: String);
    fn set_associated_layout(&mut self, id: TheId);
    fn set_size(&mut self, size: i32);
    fn set_icon(&mut self, icon: TheRGBABuffer);
}

impl TheListItemTrait for TheListItem {
    fn set_text(&mut self, text: String) {
        self.text = text;
        self.is_dirty = true;
    }
    fn set_sub_text(&mut self, sub_text: String) {
        self.sub_text = sub_text;
        self.is_dirty = true;
    }
    fn set_associated_layout(&mut self, layout_id: TheId) {
        self.layout_id = layout_id;
    }
    fn set_size(&mut self, size: i32) {
        self.limiter_mut().set_max_height(size);
        self.is_dirty = true;
    }
    fn set_icon(&mut self, icon: TheRGBABuffer) {
        self.icon = Some(icon);
    }
}
