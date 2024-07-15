use std::time::Instant;

use crate::prelude::*;

use super::thetextedit::{TheCursor, TheTextEditState, TheTextRenderer};

#[derive(Debug, PartialEq)]
pub enum TheTextLineEditContentType {
    Unknown,
    Text,
    Float,
    Int,
}

pub struct TheTextLineEdit {
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
    padding: (usize, usize, usize, usize), // left top right bottom
    renderer: TheTextRenderer,

    // Interaction
    drag_start_index: usize,
    last_mouse_down_coord: Vec2<i32>,
    last_mouse_down_time: Instant,

    // Range
    range: Option<TheValue>,
    original: String,

    is_dirty: bool,
    embedded: bool,

    layout_id: Option<TheId>,
    continuous: bool,

    content_type: TheTextLineEditContentType,
}

impl TheWidget for TheTextLineEdit {
    fn new(id: TheId) -> Self
    where
        Self: Sized,
    {
        let mut limiter = TheSizeLimiter::new();
        limiter.set_max_width(150);
        limiter.set_max_height(20);

        Self {
            id,
            limiter,
            status: None,

            dim: TheDim::zero(),

            is_disabled: false,

            state: TheTextEditState::default(),
            modified_since_last_return: false,
            modified_since_last_tick: false,

            padding: (5, 0, 5, 0),
            renderer: TheTextRenderer::default(),

            drag_start_index: 0,
            last_mouse_down_coord: Vec2::zero(),
            last_mouse_down_time: Instant::now(),

            range: None,
            original: "".to_string(),

            is_dirty: false,
            embedded: false,

            layout_id: None,
            continuous: false,

            content_type: TheTextLineEditContentType::Unknown,
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
                    self.state.set_cursor(self.find_cursor(coord));
                    self.drag_start_index = self.state.find_cursor_index();

                    if self.is_range() && self.state.selection.is_none() {
                        self.state.select_row();
                    } else {
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
                if self.is_range() {
                    self.original.clone_from(&self.state.to_text());
                }

                self.last_mouse_down_coord = *coord;
                self.last_mouse_down_time = Instant::now();
            }
            TheEvent::MouseDragged(coord) => {
                self.is_dirty = true;
                redraw = true;

                // If we have an i32 or f32 range, we treat the text line edit as a slider
                if let Some(range) = &self.range {
                    if let Some(range_f32) = range.to_range_f32() {
                        let d = abs(range_f32.end() - range_f32.start())
                            * (coord.x.as_f32() / (self.dim.width).as_f32()).clamp(0.0, 1.0);
                        let v = *range_f32.start() + d;
                        self.state.set_text(format!("{:.3}", v));
                        self.modified_since_last_tick = true;
                    } else if let Some(range_i32) = range.to_range_i32() {
                        let range_diff = range_i32.end() - range_i32.start();
                        let d = (coord.x * range_diff) / (self.dim.width);
                        let v =
                            (*range_i32.start() + d).clamp(*range_i32.start(), *range_i32.end());
                        self.state.set_text(v.to_string());
                        self.modified_since_last_tick = true;
                    }
                    if self.continuous {
                        if let Some(range_f32) = range.to_range_f32() {
                            if let Ok(v) = self.state.to_text().parse::<f32>() {
                                ctx.ui.send_widget_value_changed(
                                    self.id(),
                                    TheValue::FloatRange(v, range_f32),
                                );
                            }
                        } else if let Some(range_i32) = range.to_range_i32() {
                            if let Ok(v) = self.state.to_text().parse::<i32>() {
                                ctx.ui.send_widget_value_changed(
                                    self.id(),
                                    TheValue::IntRange(v, range_i32),
                                );
                            }
                        }
                    }
                } else if !self.state.is_empty() {
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
                            .scroll(&vec2i(delta_x / ratio, delta_y / ratio));
                    }

                    self.state.set_cursor(self.find_cursor(coord));

                    let cursor_index = self.state.find_cursor_index();
                    if self.drag_start_index != cursor_index {
                        let start = self.drag_start_index.min(cursor_index);
                        let end = self.drag_start_index.max(cursor_index);
                        self.state.select(start, end);
                    } else {
                        self.state.reset_selection();
                    }
                }
            }
            TheEvent::MouseUp(_coord) => {
                self.drag_start_index = 0;
                // Send an event if in slider mode and not continuous
                if self.is_range() && !self.continuous && self.state.to_text() != self.original {
                    if let Some(range) = &self.range {
                        if let Some(range_f32) = range.to_range_f32() {
                            if let Ok(v) = self.state.to_text().parse::<f32>() {
                                ctx.ui.send_widget_value_changed(
                                    self.id(),
                                    TheValue::FloatRange(v, range_f32),
                                );
                            }
                        } else if let Some(range_i32) = range.to_range_i32() {
                            if let Ok(v) = self.state.to_text().parse::<i32>() {
                                ctx.ui.send_widget_value_changed(
                                    self.id(),
                                    TheValue::IntRange(v, range_i32),
                                );
                            }
                        }
                    }
                }
            }
            TheEvent::MouseWheel(delta) => {
                if self.renderer.scroll(&vec2i(delta.x / 4, delta.y / 4)) {
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
                        if let Some(layout_id) = &self.layout_id {
                            ctx.ui.send(TheEvent::RedirectWidgetValueToLayout(
                                layout_id.clone(),
                                self.id().clone(),
                                self.value(),
                            ));
                        } else {
                            ctx.ui.send_widget_value_changed(self.id(), self.value());
                        }
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
                    } else if key == TheKeyCode::Return && self.modified_since_last_return {
                        if let Some(layout_id) = &self.layout_id {
                            ctx.ui.send(TheEvent::RedirectWidgetValueToLayout(
                                layout_id.clone(),
                                self.id().clone(),
                                self.value(),
                            ));
                        } else {
                            ctx.ui.send_widget_value_changed(self.id(), self.value());
                        }
                        ctx.ui.clear_focus();
                        redraw = true;
                        self.is_dirty = true;
                        self.modified_since_last_return = false;
                        if self.is_range() {
                            self.original = self.state.to_text();
                        }
                    }

                    if self.continuous {
                        if let Some(layout_id) = &self.layout_id {
                            ctx.ui.send(TheEvent::RedirectWidgetValueToLayout(
                                layout_id.clone(),
                                self.id().clone(),
                                self.value(),
                            ));
                        } else {
                            ctx.ui.send_widget_value_changed(self.id(), self.value());
                        }
                    }
                }
            }
            TheEvent::LostFocus(_id) => {
                if self.modified_since_last_return {
                    if let Some(layout_id) = &self.layout_id {
                        ctx.ui.send(TheEvent::RedirectWidgetValueToLayout(
                            layout_id.clone(),
                            self.id().clone(),
                            self.value(),
                        ));
                    } else {
                        ctx.ui.send_widget_value_changed(self.id(), self.value());
                    }
                }
            }
            TheEvent::Hover(_coord) => {
                if !self.id().equals(&ctx.ui.hover) {
                    ctx.ui.set_hover(self.id());
                }
            }
            _ => {}
        }
        redraw
    }

    fn value(&self) -> TheValue {
        if let Some(range) = &self.range {
            if let Some(range_f32) = range.to_range_f32() {
                if let Ok(value) = self.state.to_text().parse::<f32>() {
                    //if range_f32.contains(&value) {
                    return TheValue::FloatRange(value, range_f32);
                    //}
                }
                let original = self.original.clone();
                if let Ok(value) = original.parse::<f32>() {
                    if range_f32.contains(&value) {
                        return TheValue::Float(value);
                    }
                }
            } else if let Some(range_i32) = range.to_range_i32() {
                if let Ok(value) = self.state.to_text().parse::<i32>() {
                    // if range_i32.contains(&value) {
                    return TheValue::IntRange(value, range_i32);
                    // }
                }
                let original = self.original.clone();
                if let Ok(value) = original.parse::<i32>() {
                    if range_i32.contains(&value) {
                        return TheValue::Int(value);
                    }
                }
            }
        }
        if self.content_type == TheTextLineEditContentType::Float {
            if let Ok(value) = self.state.to_text().parse::<f32>() {
                return TheValue::Float(value);
            }
        }
        if self.content_type == TheTextLineEditContentType::Int {
            if let Ok(value) = self.state.to_text().parse::<i32>() {
                return TheValue::Int(value);
            }
        }
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
                self.content_type = TheTextLineEditContentType::Text;
                self.set_text(text);
            }
            TheValue::Int(v) => {
                self.content_type = TheTextLineEditContentType::Int;
                self.set_text(v.to_string());
            }
            TheValue::Float(v) => {
                self.content_type = TheTextLineEditContentType::Float;
                self.set_text(v.to_string());
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
            self.renderer.prepare(
                &self.state,
                &self.dim.to_buffer_shrunk_utuple(&shrinker),
                font,
                &ctx.draw,
            );
        }

        if self.is_range() && !self.is_disabled {
            shrinker.shrink_by(
                -self.renderer.padding.0.as_i32(),
                -self.renderer.padding.1.as_i32(),
                -self.renderer.padding.2.as_i32(),
                -self.renderer.padding.3.as_i32(),
            );
            let rect = self.dim.to_buffer_shrunk_utuple(&shrinker);
            let value = self.range.as_ref().and_then(|range| {
                if let Some(range_f32) = range.to_range_f32() {
                    if let Ok(value) = self.state.to_text().parse::<f32>() {
                        let normalized =
                            (value - range_f32.start()) / (range_f32.end() - range_f32.start());
                        return Some((normalized * rect.2.as_f32()).as_usize());
                    }
                } else if let Some(range_i32) = range.to_range_i32() {
                    if let Ok(value) = self.state.to_text().parse::<i32>() {
                        let range_diff = range_i32.end() - range_i32.start();
                        let normalized = (value - range_i32.start()) * rect.2.as_i32() / range_diff;
                        return Some(normalized.as_usize());
                    }
                }
                None
            });

            if let Some(value) = value {
                let pos = value.clamp(0, rect.2);
                let stride = buffer.stride();
                ctx.draw.rect(
                    buffer.pixels_mut(),
                    &(rect.0, rect.1, pos, rect.3),
                    stride,
                    style.theme().color(TextEditRange),
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

        self.modified_since_last_return =
            self.modified_since_last_return || self.modified_since_last_tick;
        self.modified_since_last_tick = false;
        self.is_dirty = false;
    }

    fn as_text_line_edit(&mut self) -> Option<&mut dyn TheTextLineEditTrait> {
        Some(self)
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub trait TheTextLineEditTrait: TheWidget {
    fn text(&self) -> String;
    fn set_text(&mut self, text: String);
    fn set_font_size(&mut self, font_size: f32);
    fn set_embedded(&mut self, embedded: bool);
    fn set_range(&mut self, range: TheValue);
    fn set_associated_layout(&mut self, id: TheId);
    fn set_continuous(&mut self, continuous: bool);
}

impl TheTextLineEditTrait for TheTextLineEdit {
    fn text(&self) -> String {
        self.state.to_text()
    }
    fn set_text(&mut self, text: String) {
        if let Some(range) = &self.range {
            if let Some(range) = range.to_range_f32() {
                let v = text.parse::<f32>().unwrap_or(*range.start());
                self.state.set_text(format!("{:.3}", v));
            } else if let Some(range) = range.to_range_i32() {
                let v = text.parse::<i32>().unwrap_or(*range.start());
                self.state.set_text(v.to_string());
            }
        } else {
            self.state.set_text(text);
        }
        self.content_type = TheTextLineEditContentType::Text;
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
    fn set_range(&mut self, range: TheValue) {
        if Some(range.clone()) != self.range {
            if let Some(range) = range.to_range_f32() {
                let v = self
                    .state
                    .to_text()
                    .parse::<f32>()
                    .unwrap_or(*range.start());
                self.state.set_text(format!("{:.3}", v));
                self.content_type = TheTextLineEditContentType::Float;
            } else if let Some(range) = range.to_range_i32() {
                let v = self
                    .state
                    .to_text()
                    .parse::<i32>()
                    .unwrap_or(*range.start());
                self.state.set_text(v.to_string());
                self.content_type = TheTextLineEditContentType::Int;
            }
            self.range = Some(range);
            self.is_dirty = true;
        }
    }
    fn set_associated_layout(&mut self, layout_id: TheId) {
        self.layout_id = Some(layout_id);
    }
    fn set_continuous(&mut self, continuous: bool) {
        self.continuous = continuous;
    }
}

impl TheTextLineEdit {
    fn find_cursor(&self, coord: &Vec2<i32>) -> TheCursor {
        self.renderer.find_cursor(&vec2i(
            coord.x - self.padding.0.as_i32(),
            coord.y - self.padding.1.as_i32(),
        ))
    }

    fn is_range(&self) -> bool {
        self.range.is_some()
    }
}
