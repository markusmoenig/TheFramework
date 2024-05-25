use std::{ops::Range, time::Instant};

use fontdue::Font;

use crate::prelude::*;

pub struct TheTextLineEdit {
    id: TheId,
    limiter: TheSizeLimiter,
    status: Option<String>,

    is_disabled: bool,

    text: String,
    text_last_tick: String,
    original: String,

    cursor_position: usize,
    drag_start_position: usize,
    last_mouse_down_time: Option<Instant>,
    last_mouse_down_coord: Vec2<i32>,

    selection: Option<Range<usize>>,

    font_size: f32,
    // x, width
    glyph_positions: Vec<(f32, usize)>,

    dim: TheDim,
    // left top right bottom
    padding: (i32, i32, i32, i32),
    scroll_offset: Vec2<i32>,
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
            text_last_tick: "".to_string(),
            original: "".to_string(),

            cursor_position: 0,
            drag_start_position: 0,
            last_mouse_down_time: None,
            last_mouse_down_coord: Vec2::new(0, 0),

            selection: None,

            font_size: 14.0,
            glyph_positions: vec![],

            dim: TheDim::zero(),
            padding: (5, 0, 5, 0),
            scroll_offset: Vec2::zero(),
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
                self.original.clone_from(&self.text);

                self.cursor_position = 0;
                self.selection = None;
                if !self.text.is_empty() {
                    self.cursor_position = find_cursor_position(
                        &self.text,
                        &ctx.ui.font,
                        self.font_size,
                        coord.x - self.padding.0 + self.scroll_offset.x,
                        &ctx.draw,
                    );
                    self.drag_start_position = self.cursor_position;

                    // Select all if double click
                    if let Some(last_time) = self.last_mouse_down_time {
                        if last_time.elapsed().as_millis() < 500
                            && self.last_mouse_down_coord == *coord
                        {
                            self.selection = Some(0..self.text.len());
                        }
                    }
                    self.last_mouse_down_time = Some(Instant::now());
                    self.last_mouse_down_coord = *coord;
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
                    self.cursor_position = find_cursor_position(
                        &self.text,
                        &ctx.ui.font,
                        self.font_size,
                        coord.x - self.padding.0 + self.scroll_offset.x,
                        &ctx.draw,
                    );

                    // Select all chars in the left if dragging up
                    if coord.y < 0 {
                        self.cursor_position = 0;
                    // Select all chars in the right if dragging down
                    } else if coord.y > self.dim.height {
                        self.cursor_position = self.text.len();
                    }

                    if self.drag_start_position != self.cursor_position {
                        let left = self.drag_start_position.min(self.cursor_position);
                        let right = self.drag_start_position.max(self.cursor_position);
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
                    insert_at_char_position(&mut txt, c, self.cursor_position);

                    self.text = txt;
                    self.cursor_position += 1;
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
                        if self.cursor_position > 0 {
                            delete_at_char_position(&mut self.text, self.cursor_position - 1);
                            self.cursor_position -= 1;
                            self.is_dirty = true;
                            redraw = true;
                        }
                    } else if key == TheKeyCode::Left && self.cursor_position > 0 {
                        self.cursor_position -= 1;
                        self.is_dirty = true;
                        redraw = true;
                    } else if key == TheKeyCode::Right && self.cursor_position < self.text.len() {
                        self.cursor_position += 1;
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
                        self.original.clone_from(&self.text);
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
                self.text.clone_from(&text);
                self.is_dirty = true;
            }
            TheValue::Float(v) => {
                self.text.clone_from(&v.to_string());
                self.is_dirty = true;
            }
            TheValue::Int(v) => {
                self.text.clone_from(&v.to_string());
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
            let rect = self.dim.to_buffer_shrunk_utuple(&shrinker);

            ctx.draw.rect(
                buffer.pixels_mut(),
                &rect,
                stride,
                style.theme().color(TextEditBackground),
            );

            let mut pos = None;

            if let Some(range) = &self.range {
                if let Some(range_f32) = range.to_range_f32() {
                    if let Ok(value) = self.text.parse::<f32>() {
                        let normalized =
                            (value - range_f32.start()) / (range_f32.end() - range_f32.start());
                        pos = Some((normalized * rect.2 as f32) as usize);
                    }
                } else if let Some(range_i32) = range.to_range_i32() {
                    if let Ok(value) = self.text.parse::<i32>() {
                        let range_diff = range_i32.end() - range_i32.start();
                        let normalized = (value - range_i32.start()) * rect.2 as i32 / range_diff;
                        pos = Some(normalized as usize);
                    }
                }
            }

            if let Some(mut pos) = pos {
                pos = pos.clamp(0, rect.2);
                ctx.draw.rect(
                    buffer.pixels_mut(),
                    &(rect.0, rect.1, pos, rect.3),
                    stride,
                    style.theme().color(TextEditRange),
                );
            }
        } else {
            ctx.draw.blend_rect(
                buffer.pixels_mut(),
                &self.dim.to_buffer_shrunk_utuple(&shrinker),
                stride,
                style.theme().color_disabled_t(TextEditBackground),
            );
        }

        shrinker.shrink_by(
            self.padding.0,
            self.padding.1,
            self.padding.2,
            self.padding.3,
        );

        if let Some(font) = &ctx.ui.font {
            if self.text != self.text_last_tick {
                self.glyph_positions = ctx
                    .draw
                    .get_text_layout(font, self.font_size, &self.text)
                    .glyphs()
                    .iter()
                    .map(|g| (g.x, g.width))
                    .collect::<Vec<(f32, usize)>>();
            }

            let visible_area = self.dim.to_buffer_shrunk_utuple(&shrinker);
            let text_width_before_cursor =
                get_text_left(&self.glyph_positions, self.cursor_position);

            // Check if the widget should be scrolled in order to display the cursor
            // Scroll right
            let leftmost = text_width_before_cursor as i32;
            self.scroll_offset.x = self.scroll_offset.x.min(leftmost);
            // Scroll left
            let rightmost = text_width_before_cursor as i32 - visible_area.2 as i32;
            self.scroll_offset.x = self.scroll_offset.x.max(rightmost);

            let text_start_x = visible_area.0 as i32 - self.scroll_offset.x;

            if !self.text.is_empty() {
                // Render selection
                if let Some(selection) = &self.selection {
                    let leftside_nonselection_text_width =
                        get_text_left(&self.glyph_positions, selection.start);
                    let selection_width =
                        get_text_width(&self.glyph_positions, selection.start, selection.end);

                    let highlight_area_left =
                        (text_start_x + leftside_nonselection_text_width as i32).max(0) as usize;
                    let highlight_area_left = highlight_area_left.max(visible_area.0);
                    let highlight_area_left =
                        highlight_area_left.min(visible_area.0 + visible_area.2);

                    let mut highlight_area_width = selection_width;
                    if highlight_area_left + highlight_area_width > visible_area.0 + visible_area.2
                    {
                        highlight_area_width =
                            visible_area.0 + visible_area.2 - highlight_area_left;
                    }

                    let highlight_area = (
                        highlight_area_left,
                        visible_area.1,
                        highlight_area_width,
                        visible_area.3,
                    );
                    ctx.draw.blend_rect(
                        buffer.pixels_mut(),
                        &highlight_area,
                        stride,
                        style.theme().color(TextEditSelectionBackground),
                    );
                }

                // Find the visible text
                let mut visible_text_start_position = 0;
                let mut visible_text_end_position = self.text.len();
                let mut chars_acc_width = 0;
                for i in 0..self.text.len() {
                    let is_start_position_found =
                        chars_acc_width > self.scroll_offset.x.unsigned_abs() as usize;
                    chars_acc_width = get_text_left(&self.glyph_positions, i + 1);
                    if is_start_position_found {
                        if chars_acc_width
                            >= self.scroll_offset.x.unsigned_abs() as usize + visible_area.2
                        {
                            visible_text_end_position = i + 1;
                            break;
                        }
                    } else if chars_acc_width >= self.scroll_offset.x.unsigned_abs() as usize {
                        visible_text_start_position = i;
                    }
                }
                let visible_text =
                    &self.text[visible_text_start_position..visible_text_end_position];

                // Render text and clip
                let leftside_invisible_chars_width =
                    self.glyph_positions[visible_text_start_position].0;
                let visible_text_top_left = Vec2::new(
                    (text_start_x + leftside_invisible_chars_width as i32).max(0) as usize,
                    visible_area.1 - 1,
                );
                ctx.draw.text_rect_blend_clip(
                    buffer.pixels_mut(),
                    &visible_text_top_left,
                    &visible_area,
                    stride,
                    font,
                    self.font_size,
                    visible_text,
                    style.theme().color(TextEditTextColor),
                    TheHorizontalAlign::Center,
                    TheVerticalAlign::Center,
                );
            }

            if ctx.ui.has_focus(self.id()) {
                let cursor_width = 2;
                let cursor_vertical_shrink = 1;
                let cursor_left =
                    (text_start_x + text_width_before_cursor as i32 - 1).max(0) as usize;
                if cursor_left >= visible_area.0 - cursor_width / 2
                    && cursor_left < visible_area.0 + visible_area.2
                {
                    ctx.draw.rect(
                        buffer.pixels_mut(),
                        &(
                            cursor_left,
                            visible_area.1 + cursor_vertical_shrink,
                            cursor_width,
                            visible_area.3 - cursor_vertical_shrink * 2,
                        ),
                        stride,
                        style.theme().color(TextEditCursorColor),
                    );
                }
            }
        }

        self.text_last_tick.clone_from(&self.text);
        self.is_dirty = false;

        fn get_text_left(glyphs: &[(f32, usize)], index: usize) -> usize {
            if !glyphs.is_empty() {
                if index >= glyphs.len() {
                    return glyphs[glyphs.len() - 1].0.ceil() as usize
                        + glyphs[glyphs.len() - 1].1
                        + 1;
                }
                glyphs[index].0.ceil() as usize
            } else {
                0
            }
        }

        fn get_text_width(glyphs: &[(f32, usize)], start: usize, end: usize) -> usize {
            let left = start.min(end);
            let right = start.max(end);

            if right >= glyphs.len() {
                return get_text_left(glyphs, right) - glyphs[left].0.ceil() as usize;
            }

            (glyphs[right].0 - glyphs[left].0).ceil() as usize
        }
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
        self.cursor_position = 0;
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
    draw: &TheDraw2D,
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
