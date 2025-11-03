use crate::prelude::*;

pub struct TheTreeItem {
    id: TheId,
    limiter: TheSizeLimiter,

    state: TheWidgetState,

    text: String,
    sub_text: String,

    dim: TheDim,
    is_dirty: bool,

    icon: Option<TheRGBABuffer>,
    status: Option<String>,

    layout_id: TheId,
    scroll_offset: i32,

    values: Vec<(i32, TheValue)>,
    widgets: Vec<(i32, Box<dyn TheWidget>)>,

    context_menu: Option<TheContextMenu>,

    background: Option<TheColor>,
}

impl TheWidget for TheTreeItem {
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

            text: "".to_string(),
            sub_text: "".to_string(),

            dim: TheDim::zero(),
            is_dirty: true,

            icon: None,
            status: None,

            layout_id: TheId::empty(),
            scroll_offset: 0,

            values: Vec::new(),
            widgets: Vec::new(),

            context_menu: None,

            background: None,
        }
    }

    fn id(&self) -> &TheId {
        &self.id
    }

    fn set_context_menu(&mut self, menu: Option<TheContextMenu>) {
        self.context_menu = menu;
    }

    fn on_event(&mut self, event: &TheEvent, ctx: &mut TheContext) -> bool {
        let mut redraw = false;
        // println!("event ({}): {:?}", self.widget_id.name, event);
        match event {
            TheEvent::Context(coord) => {
                if let Some(context_menu) = &self.context_menu {
                    ctx.ui.send(TheEvent::ShowContextMenu(
                        self.id().clone(),
                        *coord,
                        context_menu.clone(),
                    ));
                    ctx.ui.set_focus(self.id());
                    redraw = true;
                    self.is_dirty = true;
                }
            }
            TheEvent::MouseDown(coord) => {
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

                for (_, w) in &mut self.widgets {
                    let dim = w.dim();
                    let c = Vec2::new(coord.x - dim.x + 15, coord.y - dim.y);
                    if c.x > 0 {
                        redraw = w.on_event(
                            &TheEvent::MouseDown(Vec2::new(coord.x - dim.x + 15, coord.y - dim.y)),
                            ctx,
                        );
                    }
                }
            }
            TheEvent::MouseUp(coord) => {
                for (_, w) in &mut self.widgets {
                    let dim = w.dim();
                    redraw = w.on_event(
                        &TheEvent::MouseUp(Vec2::new(coord.x - dim.x + 15, coord.y)),
                        ctx,
                    );
                    self.is_dirty = true;
                }
            }
            TheEvent::MouseDragged(coord) => {
                for (_, w) in &mut self.widgets {
                    let dim = w.dim();
                    w.on_event(
                        &TheEvent::MouseDragged(Vec2::new(coord.x - dim.x + 15, coord.y)),
                        ctx,
                    );
                    self.is_dirty = true;
                    redraw = true;
                }
            }
            TheEvent::Hover(_coord) => {
                if self.state != TheWidgetState::Selected && !self.id().equals(&ctx.ui.hover) {
                    self.is_dirty = true;
                    ctx.ui.set_hover(self.id());
                    redraw = true;
                }
            }
            TheEvent::MouseWheel(delta) => {
                ctx.ui
                    .send(TheEvent::ScrollLayout(self.layout_id.clone(), *delta));
            }
            _ => {
                for (_, w) in &mut self.widgets {
                    redraw = w.on_event(event, ctx)
                }
            }
        }
        redraw
    }

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
                self.text.clone_from(&text);
                self.is_dirty = true;
            }
            TheValue::Image(image) => {
                self.icon = Some(image);
                self.is_dirty = true;
            }
            _ => {}
        }
    }

    fn status_text(&self) -> Option<String> {
        self.status.clone()
    }

    fn set_status_text(&mut self, text: &str) {
        self.status = Some(text.to_string());
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
                *style.theme().color(ListItemSelectedNoFocus)
            } else {
                *style.theme().color(ListItemSelected)
            }
        } else if let Some(background) = &self.background {
            background.to_u8_array()
        } else {
            *style.theme().color(ListItemNormal)
        };

        if self.state != TheWidgetState::Selected && self.id().equals(&ctx.ui.hover) {
            color = *style.theme().color(ListItemHover)
        }

        let stride = buffer.stride();
        let mut shrinker = TheDimShrinker::zero();

        ctx.draw.rect_outline_border_open(
            buffer.pixels_mut(),
            &self.dim.to_buffer_shrunk_utuple(&shrinker),
            stride,
            &color,
            1,
        );

        shrinker.shrink(1);
        ctx.draw.rect(
            buffer.pixels_mut(),
            &self.dim.to_buffer_shrunk_utuple(&shrinker),
            stride,
            &color,
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
            let mut right_width = 5;
            for v in self.values.iter() {
                right_width += v.0;
            }
            for v in self.widgets.iter() {
                right_width += v.0;
            }

            shrinker.shrink_by(9, 0, 0, 0);
            let mut rect: (usize, usize, usize, usize) =
                self.dim.to_buffer_shrunk_utuple(&shrinker);

            if let Some(font) = &ctx.ui.font {
                ctx.draw.text_rect_blend(
                    buffer.pixels_mut(),
                    &(rect.0, rect.1, rect.2 - right_width as usize, rect.3),
                    stride,
                    font,
                    13.0,
                    &self.text,
                    style.theme().color(ListItemText),
                    TheHorizontalAlign::Left,
                    TheVerticalAlign::Center,
                );
            }

            rect.0 += rect.2 - right_width as usize;

            for (width, widget) in self.widgets.iter_mut() {
                ctx.draw.rect(
                    buffer.pixels_mut(),
                    &(rect.0, rect.1 - 1, 1, rect.3 + 2),
                    stride,
                    style.theme().color(ListLayoutBackground),
                );

                widget.calculate_size(ctx);
                let mut y = rect.1 as i32;
                let height = widget.limiter().get_max_height();
                if height < 22 {
                    y += (22 - height) / 2 - 1;
                }

                widget.set_dim(
                    TheDim::new(rect.0 as i32 + 9, y, *width as i32 - 10, height),
                    ctx,
                );
                widget.dim_mut().set_buffer_offset(rect.0 as i32 + 9, y);
                widget.draw(buffer, style, ctx);
            }

            if let Some(font) = &ctx.ui.font {
                for (width, value) in self.values.iter() {
                    ctx.draw.rect(
                        buffer.pixels_mut(),
                        &(rect.0, rect.1 - 1, 1, rect.3 + 2),
                        stride,
                        style.theme().color(ListLayoutBackground),
                    );

                    #[allow(clippy::single_match)]
                    match value {
                        TheValue::Text(text) => {
                            ctx.draw.text_rect_blend(
                                buffer.pixels_mut(),
                                &(rect.0 + 9, rect.1, *width as usize - 10, rect.3),
                                stride,
                                font,
                                13.0,
                                text,
                                style.theme().color(ListItemText),
                                TheHorizontalAlign::Left,
                                TheVerticalAlign::Center,
                            );
                        }
                        _ => {
                            ctx.draw.text_rect_blend(
                                buffer.pixels_mut(),
                                &(rect.0 + 9, rect.1, *width as usize - 10, rect.3),
                                stride,
                                font,
                                13.0,
                                &value.describe(),
                                style.theme().color(ListItemText),
                                TheHorizontalAlign::Left,
                                TheVerticalAlign::Center,
                            );
                        }
                    }
                }
            }
        }

        self.is_dirty = false;
    }

    // fn as_list_item(&mut self) -> Option<&mut dyn TheListItemTrait> {
    //     Some(self)
    // }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub trait TheTreeItemTrait {
    fn set_background_color(&mut self, color: TheColor);
    fn set_text(&mut self, text: String);
    fn set_sub_text(&mut self, sub_text: String);
    fn set_associated_layout(&mut self, id: TheId);
    fn set_size(&mut self, size: i32);
    fn set_icon(&mut self, icon: TheRGBABuffer);
    fn set_scroll_offset(&mut self, offset: i32);
    fn add_value_column(&mut self, width: i32, value: TheValue);
    fn add_widget_column(&mut self, width: i32, value: Box<dyn TheWidget>);
}

impl TheTreeItemTrait for TheTreeItem {
    fn set_background_color(&mut self, color: TheColor) {
        self.background = Some(color);
        self.is_dirty = true;
    }
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
    fn set_scroll_offset(&mut self, offset: i32) {
        self.scroll_offset = offset;
    }
    fn add_value_column(&mut self, width: i32, value: TheValue) {
        self.values.push((width, value));
    }
    fn add_widget_column(&mut self, width: i32, mut widget: Box<dyn TheWidget>) {
        widget.set_embedded(true);
        self.widgets.push((width, widget));
    }
}
