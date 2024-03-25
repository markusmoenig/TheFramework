use std::ops::Range;

use fontdue::Font;

use crate::prelude::*;

pub struct TheTextLineEdit {
    id: TheId,
    limiter: TheSizeLimiter,
    status: Option<String>,

    is_disabled: bool,

    text: String,
    original: String,
    position: usize,
    drag_start_position: usize,
    selection: Option<Range<usize>>,

    font_size: f32,

    dim: TheDim,
    is_dirty: bool,
    embedded: bool,

    range: Option<TheValue>,

    layout_id: Option<TheId>,
    continuous: bool,
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

            is_disabled: false,

            text: "".to_string(),
            original: "".to_string(),
            position: 0,
            drag_start_position: 0,
            selection: None,

            font_size: 14.0,

            dim: TheDim::zero(),
            is_dirty: false,

            embedded: false,

            range: None,

            layout_id: None,
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
        let mut redraw = false;
        //println!("event ({}): {:?}", self.widget_id.name, event);
        if self.is_disabled {
            return false;
        }
        match event {
            TheEvent::MouseDown(coord) => {
                ctx.ui.set_focus(self.id());
                self.is_dirty = true;
                redraw = true;
                self.original = self.text.clone();

                self.position = 0;
                self.selection = None;
                if !self.text.is_empty() {
                    self.position = find_cursor_position(
                        &self.text,
                        &ctx.ui.font,
                        self.font_size,
                        coord.x,
                        &ctx.draw
                    );
                    self.drag_start_position = self.position;
                }
            }
            TheEvent::MouseDragged(coord) => {
                self.is_dirty = true;
                redraw = true;

                // If we have an i32 or f32 range, we treat the text line edit as a slider
                if let Some(range) = &self.range {
                    if let Some(range_f32) = range.to_range_f32() {
                        let d = abs(range_f32.end() - range_f32.start())
                            * (coord.x as f32 / (self.dim.width) as f32).clamp(0.0, 1.0);
                        let v = *range_f32.start() + d;
                        self.text = format!("{:.3}", v);
                    } else if let Some(range_i32) = range.to_range_i32() {
                        let range_diff = range_i32.end() - range_i32.start();
                        let d = (coord.x * range_diff) / (self.dim.width);
                        let v =
                            (*range_i32.start() + d).clamp(*range_i32.start(), *range_i32.end());
                        self.text = v.to_string()
                    }
                    if self.continuous {
                        ctx.ui.send_widget_value_changed(
                            self.id(),
                            TheValue::Text(self.text.clone()),
                        );
                    }
                } else if !self.text.is_empty() {
                    self.position = find_cursor_position(
                        &self.text,
                        &ctx.ui.font,
                        self.font_size,
                        coord.x,
                        &ctx.draw
                    );

                    // Select all chars in the left if dragging up
                    if coord.y < 0 {
                        self.position = 0;
                    // Select all chars in the right if dragging down
                    } else if coord.y > self.dim.height {
                            self.position = self.text.len();
                        }

                    if self.drag_start_position != self.position {
                        let left = self.drag_start_position.min(self.position);
                        let right = self.drag_start_position.max(self.position);
                        self.selection = Some(left..right);
                    } else {
                        self.selection = None;
                    }
                }
            }
            TheEvent::MouseUp(_coord) => {
                self.drag_start_position = 0;
            }
            TheEvent::KeyDown(key) => {
                if let Some(c) = key.to_char() {
                    fn insert_at_char_position(s: &mut String, ch: char, pos: usize) {
                        // Convert the character position to a byte position
                        let byte_pos = s
                            .char_indices()
                            .nth(pos)
                            .map(|(idx, _)| idx)
                            .unwrap_or_else(|| s.len()); // If position is out of range, insert at the end

                        // Insert the character
                        s.insert(byte_pos, ch);
                    }

                    let mut txt = self.text.clone();
                    insert_at_char_position(&mut txt, c, self.position);

                    // For now limit the input to the available widget width
                    // Have to implement scrolling
                    if let Some(font) = &ctx.ui.font {
                        let size = ctx.draw.get_text_size(font, self.font_size, txt.as_str());
                        if (size.0 as i32) < self.dim().width - 12 {
                            self.text = txt;
                            self.position += 1;
                            self.is_dirty = true;
                            redraw = true;
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
            TheEvent::KeyCodeDown(key_code) => {
                if let Some(key) = key_code.to_key_code() {
                    if key == TheKeyCode::Delete {
                        fn delete_at_char_position(s: &mut String, pos: usize) {
                            // Find the start byte position of the character at the given position
                            if let Some((start, ch)) = s.char_indices().nth(pos) {
                                // Calculate the end byte position of the character
                                let end = start + ch.len_utf8();

                                // Reconstruct the string without the character at the given position
                                let remaining = s.split_off(end);
                                s.truncate(start);
                                s.push_str(&remaining);
                            }
                        }
                        if self.position > 0 {
                            delete_at_char_position(&mut self.text, self.position - 1);
                            self.position -= 1;
                            self.is_dirty = true;
                            redraw = true;
                        }
                    } else if key == TheKeyCode::Left && self.position > 0 {
                        self.position -= 1;
                        self.is_dirty = true;
                        redraw = true;
                    } else if key == TheKeyCode::Right && self.position < self.text.len() {
                        self.position += 1;
                        self.is_dirty = true;
                        redraw = true;
                    } else if key == TheKeyCode::Return && self.text != self.original {
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
                        self.original = self.text.clone();
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
                if self.text != self.original {
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
                if let Ok(value) = self.text.parse::<f32>() {
                    if range_f32.contains(&value) {
                        return TheValue::Float(value);
                    }
                }
                let original = self.original.clone();
                if let Ok(value) = original.parse::<f32>() {
                    if range_f32.contains(&value) {
                        return TheValue::Float(value);
                    }
                }
            } else if let Some(range_i32) = range.to_range_i32() {
                if let Ok(value) = self.text.parse::<i32>() {
                    if range_i32.contains(&value) {
                        return TheValue::Int(value);
                    }
                }
                let original = self.original.clone();
                if let Ok(value) = original.parse::<i32>() {
                    if range_i32.contains(&value) {
                        return TheValue::Int(value);
                    }
                }
            }
        }
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

        let stride = buffer.stride();
        let mut shrinker = TheDimShrinker::zero();
        let embedded = self.embedded;
        let disabled = self.is_disabled;

        style.draw_text_edit_border(buffer, self, &mut shrinker, ctx, embedded, disabled);

        if !self.is_disabled {
            ctx.draw.rect(
                buffer.pixels_mut(),
                &self.dim.to_buffer_shrunk_utuple(&shrinker),
                stride,
                style.theme().color(TextEditBackground),
            );
        } else {
            ctx.draw.blend_rect(
                buffer.pixels_mut(),
                &self.dim.to_buffer_shrunk_utuple(&shrinker),
                stride,
                style.theme().color_disabled_t(TextEditBackground),
            );
        }

        shrinker.shrink_by(5, 0, 5, 0);

        if let Some(font) = &ctx.ui.font {
            if !self.text.is_empty() {
                if let Some(selection) = &self.selection {
                    let text_width = ctx.draw.get_text_size(font, self.font_size, &self.text).0;

                    let left_spacing = if selection.start == 0 {
                        0
                    } else {
                        let txt = &self.text[..selection.start];
                        ctx.draw.get_text_size(font, self.font_size, txt).0
                    };

                    let right_spacing = if selection.end == self.text.len() {
                        0
                    } else {
                        let txt = &self.text[selection.end..];
                        ctx.draw.get_text_size(font, self.font_size, txt).0
                    };

                    let mut rect = self.dim.to_buffer_shrunk_utuple(&shrinker);
                    rect.0 += left_spacing;
                    rect.2 = text_width - left_spacing - right_spacing;

                    ctx.draw.blend_rect(
                        buffer.pixels_mut(),
                        &rect,
                        stride,
                        style.theme().color(SelectedTextEditBorder1)
                    );
                }

                let r = self.dim.to_buffer_shrunk_utuple(&shrinker);
                ctx.draw.text_blend(
                    buffer.pixels_mut(),
                    &(r.0, r.1 - 1),
                    stride,
                    font,
                    self.font_size,
                    &self.text,
                    style.theme().color(TextEditTextColor),
                );
            }

            if ctx.ui.has_focus(self.id()) {
                let mut shr = shrinker;
                shr.shrink_by(0, 1, 0, 1);
                let mut r = self.dim.to_buffer_shrunk_utuple(&shr);
                r.2 = 2;

                if !self.text.is_empty() && self.position > 0 {
                    let txt = &self.text[0..self.position];
                    let size = ctx.draw.get_text_size(font, self.font_size, txt);
                    r.0 += size.0;
                }

                ctx.draw.rect(
                    buffer.pixels_mut(),
                    &r,
                    stride,
                    style.theme().color(TextEditCursorColor),
                );
            }
        }

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
        self.text.clone()
    }
    fn set_text(&mut self, text: String) {
        self.text = text;
        self.position = 0;
        self.is_dirty = true;
    }
    fn set_font_size(&mut self, font_size: f32) {
        self.font_size = font_size;
    }
    fn set_embedded(&mut self, embedded: bool) {
        self.embedded = embedded;
    }
    fn set_range(&mut self, range: TheValue) {
        if Some(range.clone()) != self.range {
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

fn find_cursor_position(
    text: &str,
    font: &Option<Font>,
    font_size: f32,
    coord: i32,
    draw: &TheDraw2D
) -> usize {
    if coord < 0 {
        return 0;
    }

    let mut offset = 0;
    let mut found = false;
    for i in 1..text.len() {
        let txt = &text[0..i];
        if let Some(font) = font {
            let size = draw.get_text_size(font, font_size, txt);
            if size.0 as i32 >= coord {
                offset = i;
                found = true;
                break;
            }
        }
    }
    if found {
        offset
    } else {
        text.len()
    }
}