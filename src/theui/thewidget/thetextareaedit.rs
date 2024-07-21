use std::time::Instant;

use crate::prelude::*;

use super::thetextedit::{TheTextEditState, TheTextRenderer};

pub struct TheTextAreaEdit {
    // Widget Basic
    id: TheId,
    limiter: TheSizeLimiter,
    status: Option<String>,

    // Dimension
    dim: TheDim,

    // Edit State
    is_disabled: bool,

    // Text state
    state: TheTextEditState,
    modified_since_last_return: bool,
    modified_since_last_tick: bool,

    // Text render
    renderer: TheTextRenderer,
    scrollbar_size: usize,
    tab_spaces: usize,

    // Interaction
    drag_start_index: usize,
    hover_coord: Vec2<i32>,
    last_mouse_down_coord: Vec2<i32>,
    last_mouse_down_time: Instant,

    // Scrollbar
    hscrollbar: Box<dyn TheWidget>,
    vscrollbar: Box<dyn TheWidget>,
    is_hscrollbar_clicked: bool,
    is_hscrollbar_hovered: bool,
    is_vscrollbar_clicked: bool,
    is_vscrollbar_hovered: bool,

    is_dirty: bool,
    embedded: bool,

    continuous: bool,
}

impl TheWidget for TheTextAreaEdit {
    fn new(id: TheId) -> Self
    where
        Self: Sized,
    {
        let mut limiter = TheSizeLimiter::new();
        limiter.set_max_width(200);
        limiter.set_max_height(300);

        let hscrollbar = Box::new(TheHorizontalScrollbar::new(TheId::named(
            (id.name.clone() + " Horizontal Scrollbar").as_str(),
        )));
        let vscrollbar = Box::new(TheVerticalScrollbar::new(TheId::named(
            (id.name.clone() + " Vertical Scrollbar").as_str(),
        )));

        Self {
            id,
            limiter,
            status: None,

            dim: TheDim::zero(),

            is_disabled: false,

            state: TheTextEditState::default(),
            modified_since_last_return: false,
            modified_since_last_tick: false,

            renderer: TheTextRenderer::default(),
            scrollbar_size: 13,
            tab_spaces: 4,

            drag_start_index: 0,
            hover_coord: Vec2::zero(),
            last_mouse_down_coord: Vec2::zero(),
            last_mouse_down_time: Instant::now(),

            hscrollbar,
            vscrollbar,
            is_hscrollbar_clicked: false,
            is_hscrollbar_hovered: false,
            is_vscrollbar_clicked: false,
            is_vscrollbar_hovered: false,

            is_dirty: false,
            embedded: false,

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

    fn disabled(&self) -> bool {
        self.is_disabled
    }

    fn set_disabled(&mut self, disabled: bool) {
        if disabled != self.is_disabled {
            self.is_disabled = disabled;
            self.is_dirty = true;
        }
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

    fn supports_hover(&mut self) -> bool {
        true
    }

    #[allow(clippy::single_match)]
    fn on_event(&mut self, event: &TheEvent, ctx: &mut TheContext) -> bool {
        if self.is_disabled {
            return false;
        }

        let mut redraw = false;
        match event {
            TheEvent::MouseDown(coord) => {
                if !self.state.is_empty() {
                    let global_coord = coord + self.dim.screen_coord();
                    if self.renderer.is_horizontal_overflow()
                        && self.hscrollbar.dim().contains(global_coord)
                    {
                        self.is_hscrollbar_clicked = true;
                        self.hscrollbar.on_event(
                            &TheEvent::MouseDown(self.hscrollbar.dim().to_local(global_coord)),
                            ctx,
                        );
                    } else if self.renderer.is_vertical_overflow()
                        && self.vscrollbar.dim().contains(global_coord)
                    {
                        self.is_vscrollbar_clicked = true;
                        self.vscrollbar.on_event(
                            &TheEvent::MouseDown(self.vscrollbar.dim().to_local(global_coord)),
                            ctx,
                        );
                    } else if self.renderer.dim().contains(global_coord) {
                        self.state.set_cursor(self.renderer.find_cursor(coord));
                        self.drag_start_index = self.state.find_cursor_index();

                        let is_double_click = self.last_mouse_down_time.elapsed().as_millis() < 500
                            && self.last_mouse_down_coord == *coord;

                        if !self.state.selection.is_none() {
                            if is_double_click {
                                if self.state.is_row_all_selected(self.state.cursor.row) {
                                    self.state.reset_selection();
                                } else {
                                    self.state.select_row();
                                }
                            } else {
                                self.state.reset_selection();
                            }
                        } else if is_double_click {
                            // Select a word, a whole row or a spacing etc.
                            self.state.quick_select();
                        }
                    }
                }

                ctx.ui.set_focus(self.id());
                self.is_dirty = true;
                redraw = true;

                self.last_mouse_down_coord = *coord;
                self.last_mouse_down_time = Instant::now();
            }
            TheEvent::MouseDragged(coord) => {
                self.is_dirty = true;

                if !self.state.is_empty() {
                    if self.is_hscrollbar_clicked {
                        redraw = self.hscrollbar.on_event(
                            &TheEvent::MouseDragged(
                                self.hscrollbar
                                    .dim()
                                    .to_local(coord + self.dim.screen_coord()),
                            ),
                            ctx,
                        );
                        if let Some(scrollbar) = self.hscrollbar.as_horizontal_scrollbar() {
                            redraw = self.renderer.scroll(
                                &vec2i(
                                    scrollbar.scroll_offset()
                                        - self.renderer.scroll_offset.x.as_i32(),
                                    0,
                                ),
                                false,
                            ) || redraw;
                        }
                    } else if self.is_vscrollbar_clicked {
                        redraw = self.vscrollbar.on_event(
                            &TheEvent::MouseDragged(
                                self.vscrollbar
                                    .dim()
                                    .to_local(coord + self.dim.screen_coord()),
                            ),
                            ctx,
                        );
                        if let Some(scrollbar) = self.vscrollbar.as_vertical_scrollbar() {
                            redraw = self.renderer.scroll(
                                &vec2i(
                                    0,
                                    scrollbar.scroll_offset()
                                        - self.renderer.scroll_offset.y.as_i32(),
                                ),
                                false,
                            ) || redraw;
                        }
                    } else {
                        let delta_x = if coord.x < 0 {
                            coord.x
                        } else if coord.x > self.dim.width {
                            coord.x - self.dim.width
                        } else {
                            0
                        };

                        let delta_y = if coord.y < 0 {
                            coord.y
                        } else if coord.y > self.dim.height {
                            coord.y - self.dim.height
                        } else {
                            0
                        };

                        if delta_x != 0 || delta_y != 0 {
                            let ratio = if self.last_mouse_down_time.elapsed().as_millis() > 500 {
                                8
                            } else {
                                4
                            };
                            self.renderer
                                .scroll(&vec2i(delta_x / ratio, delta_y / ratio), true);
                        }

                        self.state.set_cursor(self.renderer.find_cursor(coord));

                        let cursor_index = self.state.find_cursor_index();
                        if self.drag_start_index != cursor_index {
                            let start = self.drag_start_index.min(cursor_index);
                            let end = self.drag_start_index.max(cursor_index);
                            self.state.select(start, end);
                        } else {
                            self.state.reset_selection();
                        }

                        redraw = true;
                    }
                }
            }
            TheEvent::MouseUp(coord) => {
                if self.is_hscrollbar_clicked {
                    self.hscrollbar.on_event(
                        &TheEvent::MouseUp(
                            self.hscrollbar
                                .dim()
                                .to_local(coord + self.dim.screen_coord()),
                        ),
                        ctx,
                    );
                } else if self.is_vscrollbar_clicked {
                    self.vscrollbar.on_event(
                        &TheEvent::MouseUp(
                            self.vscrollbar
                                .dim()
                                .to_local(coord + self.dim.screen_coord()),
                        ),
                        ctx,
                    );
                }

                redraw = true;
                self.is_hscrollbar_clicked = false;
                self.is_vscrollbar_clicked = false;
                self.drag_start_index = 0;
            }
            TheEvent::MouseWheel(delta) => {
                let global_coord = self.hover_coord + self.dim.screen_coord();
                let scrolled = if self.hscrollbar.dim().contains(global_coord) {
                    let delta = if delta.x.abs() > delta.y.abs() {
                        delta.x / 4
                    } else {
                        delta.y / 4
                    };
                    self.renderer.scroll(&vec2i(delta, 0), false)
                } else if self.vscrollbar.dim().contains(global_coord) {
                    let delta = if delta.x.abs() > delta.y.abs() {
                        delta.x / 4
                    } else {
                        delta.y / 4
                    };
                    self.renderer.scroll(&vec2i(0, delta), false)
                } else {
                    self.renderer
                        .scroll(&vec2i(delta.x / 4, delta.y / 4), false)
                };
                if scrolled {
                    self.is_dirty = true;
                    redraw = true;
                }
            }
            TheEvent::KeyDown(key) => {
                if let Some(c) = key.to_char() {
                    self.state.insert_char(c);
                    self.modified_since_last_tick = true;
                    self.is_dirty = true;
                    redraw = true;

                    if self.continuous {
                        self.emit_value_changed(ctx);
                    }
                }
            }
            TheEvent::KeyCodeDown(key_code) => {
                if let Some(key) = key_code.to_key_code() {
                    if key == TheKeyCode::Delete {
                        if self.state.delete_text() {
                            self.modified_since_last_tick = true;
                            self.is_dirty = true;
                            redraw = true;
                        }
                    } else if key == TheKeyCode::Return {
                        self.state.insert_row();
                        self.modified_since_last_tick = true;
                        self.is_dirty = true;
                        redraw = true;
                    } else if key == TheKeyCode::Tab {
                        self.state.insert_text(" ".repeat(self.tab_spaces));
                        self.modified_since_last_tick = true;
                        self.is_dirty = true;
                        redraw = true;
                    } else if key == TheKeyCode::Down {
                        if self.state.move_cursor_down() {
                            self.renderer.scroll_to_cursor(
                                self.state.find_cursor_index(),
                                self.state.cursor.row,
                            );
                            self.is_dirty = true;
                            redraw = true;
                        }
                    } else if key == TheKeyCode::Left {
                        if self.state.move_cursor_left() {
                            self.renderer.scroll_to_cursor(
                                self.state.find_cursor_index(),
                                self.state.cursor.row,
                            );
                            self.is_dirty = true;
                            redraw = true;
                        }
                    } else if key == TheKeyCode::Right {
                        if self.state.move_cursor_right() {
                            self.renderer.scroll_to_cursor(
                                self.state.find_cursor_index(),
                                self.state.cursor.row,
                            );
                            self.is_dirty = true;
                            redraw = true;
                        }
                    } else if key == TheKeyCode::Up && self.state.move_cursor_up() {
                        self.renderer.scroll_to_cursor(
                            self.state.find_cursor_index(),
                            self.state.cursor.row,
                        );
                        self.is_dirty = true;
                        redraw = true;
                    }
                }
            }
            TheEvent::LostFocus(_id) => {
                if self.modified_since_last_return {
                    self.emit_value_changed(ctx);
                }
            }
            TheEvent::Hover(coord) => {
                // The hovered widget is always current widget not scrollbars
                // We should manually draw hovered style to scrollbar hovered
                let global_coord = coord + self.dim.screen_coord();
                if self.renderer.is_horizontal_overflow() {
                    self.hscrollbar.on_event(
                        &TheEvent::Hover(self.hscrollbar.dim().to_local(global_coord)),
                        ctx,
                    );

                    self.is_hscrollbar_hovered = self.hscrollbar.id().equals(&ctx.ui.hover);
                    redraw = redraw || self.hscrollbar.needs_redraw();
                }
                if self.renderer.is_vertical_overflow() {
                    self.vscrollbar.on_event(
                        &TheEvent::Hover(self.vscrollbar.dim().to_local(global_coord)),
                        ctx,
                    );

                    self.is_vscrollbar_hovered = self.vscrollbar.id().equals(&ctx.ui.hover);
                    redraw = redraw || self.vscrollbar.needs_redraw();
                }

                if !self.id().equals(&ctx.ui.hover) {
                    ctx.ui.set_hover(self.id());
                    redraw = true;
                }

                self.hover_coord = *coord;
            }
            _ => {}
        }
        redraw
    }

    fn value(&self) -> TheValue {
        TheValue::Text(self.state.to_text())
    }

    fn set_value(&mut self, value: TheValue) {
        match value {
            TheValue::Empty => {
                self.state.reset();
                self.modified_since_last_tick = true;
                self.is_dirty = true;
            }
            TheValue::Text(text) => {
                self.state.set_text(text);
                self.modified_since_last_tick = true;
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
        if !self.dim.is_valid() || ctx.ui.font.is_none() {
            return;
        }

        let mut shrinker = TheDimShrinker::zero();
        self.renderer.render_widget(
            &mut shrinker,
            self.is_disabled,
            self.embedded,
            self,
            buffer,
            style,
            ctx,
        );

        let font = ctx.ui.font.as_ref().unwrap();
        if self.modified_since_last_tick || self.renderer.row_count() == 0 {
            shrinker.shrink_by(
                -self.renderer.padding.0.as_i32(),
                -self.renderer.padding.1.as_i32(),
                -self.renderer.padding.2.as_i32(),
                -self.renderer.padding.3.as_i32(),
            );
            let outer_area = self.dim.to_buffer_shrunk_utuple(&shrinker);
            shrinker.shrink_by(
                self.renderer.padding.0.as_i32(),
                self.renderer.padding.1.as_i32(),
                self.renderer.padding.2.as_i32(),
                self.renderer.padding.3.as_i32(),
            );

            let mut visible_area = self.dim.to_buffer_shrunk_utuple(&shrinker);
            self.renderer
                .prepare_glyphs(&self.state.to_text(), font, &ctx.draw);

            let is_hoverflow = self.renderer.is_horizontal_overflow();
            let is_voverflow = self.renderer.is_vertical_overflow();
            if is_hoverflow {
                visible_area.3 = visible_area.3.saturating_sub(self.scrollbar_size);
            }
            if is_voverflow {
                visible_area.2 = visible_area.2.saturating_sub(self.scrollbar_size);
            }
            self.renderer.set_dim(
                visible_area.0,
                visible_area.1,
                visible_area.2,
                visible_area.3,
            );
            self.renderer
                .scroll_to_cursor(self.state.find_cursor_index(), self.state.cursor.row);

            if is_hoverflow {
                self.hscrollbar.set_dim(TheDim::new(
                    outer_area.0.as_i32(),
                    (outer_area.1 + outer_area.3)
                        .saturating_sub(self.scrollbar_size)
                        .as_i32(),
                    outer_area
                        .2
                        .saturating_sub(if is_voverflow { self.scrollbar_size } else { 0 })
                        .as_i32(),
                    self.scrollbar_size.as_i32(),
                ));
                self.hscrollbar.dim_mut().set_buffer_offset(
                    outer_area.0.as_i32(),
                    (outer_area.1 + outer_area.3)
                        .saturating_sub(self.scrollbar_size)
                        .as_i32(),
                );
            }
            if let Some(scrollbar) = self.hscrollbar.as_horizontal_scrollbar() {
                scrollbar.set_total_width(
                    (self.renderer.actual_size.x
                        + self.renderer.padding.0
                        + self.renderer.padding.2)
                        .as_i32(),
                );
            }

            if is_voverflow {
                self.vscrollbar.set_dim(TheDim::new(
                    (outer_area.0 + outer_area.2)
                        .saturating_sub(self.scrollbar_size)
                        .as_i32(),
                    outer_area.1.as_i32(),
                    self.scrollbar_size.as_i32(),
                    outer_area
                        .3
                        .saturating_sub(if is_hoverflow { self.scrollbar_size } else { 0 })
                        .as_i32(),
                ));
                self.vscrollbar.dim_mut().set_buffer_offset(
                    (outer_area.0 + outer_area.2)
                        .saturating_sub(self.scrollbar_size)
                        .as_i32(),
                    outer_area.1.as_i32(),
                );
            }
            if let Some(scrollbar) = self.vscrollbar.as_vertical_scrollbar() {
                scrollbar.set_total_height(
                    (self.renderer.actual_size.y
                        + self.renderer.padding.1
                        + self.renderer.padding.3)
                        .as_i32(),
                );
            }
        }

        self.renderer.render_text(
            &self.state,
            ctx.ui.has_focus(self.id()),
            buffer,
            style,
            font,
            &ctx.draw,
        );

        if self.renderer.is_horizontal_overflow() {
            if let Some(scrollbar) = self.hscrollbar.as_horizontal_scrollbar() {
                scrollbar.set_scroll_offset(self.renderer.scroll_offset.x.as_i32());

                if self.is_hscrollbar_hovered {
                    ctx.ui.set_hover(self.hscrollbar.id());
                }
                self.hscrollbar.draw(buffer, style, ctx);
                if self.is_hscrollbar_hovered {
                    ctx.ui.set_hover(self.id());
                }
            }
        }
        if self.renderer.is_vertical_overflow() {
            if let Some(scrollbar) = self.vscrollbar.as_vertical_scrollbar() {
                scrollbar.set_scroll_offset(self.renderer.scroll_offset.y.as_i32());

                if self.is_vscrollbar_hovered {
                    ctx.ui.set_hover(self.vscrollbar.id());
                }
                self.vscrollbar.draw(buffer, style, ctx);
                if self.is_vscrollbar_hovered {
                    ctx.ui.set_hover(self.id());
                }
            }
        }

        self.modified_since_last_return =
            self.modified_since_last_return || self.modified_since_last_tick;
        self.modified_since_last_tick = false;
        self.is_dirty = false;
    }

    fn as_text_area_edit(&mut self) -> Option<&mut dyn TheTextAreaEditTrait> {
        Some(self)
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub trait TheTextAreaEditTrait: TheWidget {
    fn text(&self) -> String;
    fn set_text(&mut self, text: String);
    fn set_font_size(&mut self, font_size: f32);
    fn set_embedded(&mut self, embedded: bool);
    fn set_continuous(&mut self, continuous: bool);
    fn set_code_type(&mut self, code_type: &str);
    fn set_code_theme(&mut self, code_theme: &str);
}

impl TheTextAreaEditTrait for TheTextAreaEdit {
    fn text(&self) -> String {
        self.state.to_text()
    }
    fn set_text(&mut self, text: String) {
        self.state.set_text(text);
        self.modified_since_last_tick = true;
        self.is_dirty = true;
    }
    fn set_font_size(&mut self, font_size: f32) {
        self.renderer.set_font_size(font_size);
        self.modified_since_last_tick = true;
        self.is_dirty = true;
    }
    fn set_embedded(&mut self, embedded: bool) {
        self.embedded = embedded;
    }
    fn set_continuous(&mut self, continuous: bool) {
        self.continuous = continuous;
    }
    fn set_code_type(&mut self, code_type: &str) {
        self.renderer.set_code_type(code_type);
        self.modified_since_last_tick = true;
        self.is_dirty = true;
    }
    fn set_code_theme(&mut self, code_theme: &str) {
        self.renderer.set_code_theme(code_theme);
        self.modified_since_last_tick = true;
        self.is_dirty = true;
    }
}

impl TheTextAreaEdit {
    fn emit_value_changed(&mut self, ctx: &mut TheContext) {
        ctx.ui.send_widget_value_changed(self.id(), self.value());
        self.modified_since_last_return = false;
    }
}
