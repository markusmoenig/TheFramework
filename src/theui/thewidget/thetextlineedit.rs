use std::time::Instant;

use fontdue::{layout::GlyphPosition, Font};

use crate::prelude::*;

#[derive(Default, Debug, PartialEq)]
struct TheCursor {
    pub row: usize,
    pub column: usize,
}

impl TheCursor {
    pub fn zero() -> Self {
        Self::default()
    }

    pub fn is_zero(&self) -> bool {
        self.row == 0 && self.column == 0
    }

    pub fn reset(&mut self) {
        self.row = 0;
        self.column = 0;
    }
}

#[derive(Default, Debug)]
struct TheSelection {
    pub start: usize,
    pub end: usize,
}

impl TheSelection {
    pub fn intersects(&self, start: usize, end: usize) -> bool {
        start < self.end && end > self.start
    }

    pub fn is_none(&self) -> bool {
        self.start == self.end
    }

    pub fn reset(&mut self) {
        self.start = 0;
        self.end = 0;
    }
}

// cursor index  0   1   2   3   4
//  glyph index    0   1   2   3
//       cursor  |   |   |   |   |
//         text    a   b   c   \n
struct TheTextEditState {
    // Use cursor index
    cursor: TheCursor,
    // Linebreak is not stored here
    rows: Vec<String>,
    // Use cursor index
    selection: TheSelection,
}

impl Default for TheTextEditState {
    fn default() -> Self {
        Self {
            cursor: TheCursor::default(),
            rows: vec![String::default()],
            selection: TheSelection::default(),
        }
    }
}

impl TheTextEditState {
    pub fn delete_text(&mut self) -> bool {
        let deleted = if !self.selection.is_none() {
            self.delete_text_by_selection()
        } else {
            self.delete_char_by_cursor()
        };

        if self.rows.is_empty() {
            self.insert_row();
        }

        deleted
    }

    // Position of cursor in cursor index
    pub fn find_cursor_index(&self) -> usize {
        self.find_start_index_of_row(self.cursor.row) + self.cursor.column
    }

    // Range of row in cursor index
    // cursor index  0   1   2   3   4
    //         text    a   b   c   \n
    //        range  (0, 4)
    // cursor index  4   5   6   7   8
    //         text    d   e   f   \n
    //        range  (4, 8)
    pub fn find_range_of_row(&self, row_number: usize) -> (usize, usize) {
        let start = self.find_start_index_of_row(row_number);
        let end = start + self.row_len(row_number);
        (start, end)
    }

    // Range of selected glyphs within a row
    pub fn find_selected_range_of_row(&self, row_number: usize) -> Option<(usize, usize)> {
        if self.selection.is_none() {
            return None;
        }

        let (start, end) = self.find_range_of_row(row_number);
        if !self.selection.intersects(start, end) {
            return None;
        }

        // Select the linebreak only
        if self.selection.start == end - 1 {
            return Some((end - 1, end));
        }

        let left = self.selection.start.max(start);
        let right = self.selection.end.min(
            // If it's an empty row, we select the linebreak
            // Or if it's the last row
            if start + 1 == end || self.is_last_row(row_number) {
                end
            } else {
                // Eliminate the linebreak if the row is not empty,
                // and it's not the last row
                end - 1
            },
        );
        if left == right {
            None
        } else {
            Some((left, right))
        }
    }

    // Start position of row in cursor index
    pub fn find_start_index_of_row(&self, row_number: usize) -> usize {
        let mut index = 0;
        for i in 0..row_number {
            index += self.row_len(i)
        }
        index
    }

    pub fn insert_char(&mut self, char: char) {
        if !self.selection.is_none() {
            self.delete_text_by_selection();
        }

        self.rows[self.cursor.row].insert(self.cursor.column, char);
        self.move_cursor_right();
    }

    pub fn insert_row(&mut self) {
        if !self.selection.is_none() {
            self.delete_text_by_selection();
        }

        // Insert at current row
        if self.cursor.column == 0 {
            self.rows.insert(self.cursor.row, String::default());
        // Insert at next row
        } else if self.cursor.column >= self.rows[self.cursor.row].len() {
            self.rows.insert(self.cursor.row + 1, String::default());
        // Insert inside current row
        } else {
            let new_text = self.rows[self.cursor.row].split_off(self.cursor.column);
            self.rows.insert(self.cursor.row + 1, new_text);
        }

        self.cursor.column = 0;
        self.move_cursor_down();
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty() || (self.rows.len() == 1 && self.rows[0].is_empty())
    }

    pub fn is_last_row(&self, row_number: usize) -> bool {
        row_number == self.row_count() - 1
    }

    pub fn is_row_all_selected(&self, row_number: usize) -> bool {
        self.find_selected_range_of_row(row_number)
            .map_or(false, |selected_range| {
                let range = self.find_range_of_row(row_number);
                range.0 == selected_range.0 && range.1 == selected_range.1 + 1
            })
    }

    pub fn move_cursor_down(&mut self) -> bool {
        if self.is_last_row(self.cursor.row) {
            return false;
        }

        self.cursor.row += 1;
        self.cursor.column = self.cursor.column.min(self.rows[self.cursor.row].len());
        true
    }

    pub fn move_cursor_left(&mut self) -> bool {
        if self.cursor.is_zero() {
            return false;
        }

        if self.cursor.column == 0 {
            self.cursor.row -= 1;
            self.cursor.column = self.rows[self.cursor.row].len();
        } else {
            self.cursor.column -= 1;
        }
        true
    }

    pub fn move_cursor_right(&mut self) -> bool {
        if self.is_last_row(self.cursor.row)
            && self.cursor.column == self.rows[self.cursor.row].len()
        {
            return false;
        }

        if self.cursor.column == self.rows[self.cursor.row].len() {
            self.cursor.row += 1;
            self.cursor.column = 0;
        } else {
            self.cursor.column += 1;
        }
        true
    }

    pub fn move_cursor_up(&mut self) -> bool {
        if self.cursor.row == 0 {
            return false;
        }

        self.cursor.row -= 1;
        self.cursor.column = self.cursor.column.min(self.rows[self.cursor.row].len());
        true
    }

    pub fn quick_select(&mut self) {
        let text = &self.rows[self.cursor.row];
        let (row_start, row_end) = self.find_range_of_row(self.cursor.row);

        // Cursor is at the end of the row
        if self.cursor.column >= text.len() {
            // Select the linebreak of previous row
            if self.is_last_row(self.cursor.row) {
                if text.is_empty() {
                    self.selection.start = row_start - 1;
                    self.selection.end = row_start;
                    self.move_cursor_left();
                    return;
                }
            // Select the linebreak at the end of row
            } else {
                self.selection.start = row_end - 1;
                self.selection.end = row_end;
                return;
            }
        }

        // Select the empty space
        let col = self.cursor.column.min(text.len() - 1);
        let (start, end) = if text.chars().nth(col).unwrap().is_whitespace() {
            find_range(text, col, |char| !char.is_whitespace())
        }
        // Select a word or the entire row
        else {
            find_range(text, col, |char| char.is_whitespace())
        };

        self.selection.start = row_start + start;
        self.selection.end = row_start + end;

        fn find_range<P>(text: &str, index: usize, predicate: P) -> (usize, usize)
        where
            P: Fn(char) -> bool,
        {
            let start = text[..index]
                .char_indices()
                .rev()
                .find(|&(_, c)| predicate(c))
                .map_or(0, |(i, _)| i + 1);

            let end = text[index + 1..]
                .char_indices()
                .find(|&(_, c)| predicate(c))
                .map_or(text.len(), |(i, _)| index + 1 + i);

            (start, end)
        }
    }

    pub fn reset(&mut self) {
        self.rows = vec![String::default()];
        self.reset_cursor();
        self.reset_selection();
    }

    pub fn reset_cursor(&mut self) {
        self.cursor.reset();
    }

    pub fn reset_selection(&mut self) {
        self.selection.reset();
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    pub fn select(&mut self, start: usize, end: usize) {
        self.selection.start = start;
        self.selection.end = end;
    }

    pub fn select_row(&mut self) {
        let (start, end) = self.find_range_of_row(self.cursor.row);
        self.select(start, end);
    }

    pub fn set_cursor(&mut self, cursor: TheCursor) {
        self.cursor = cursor;
    }

    pub fn set_text(&mut self, text: String) {
        self.rows = text
            .split('\n')
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
    }

    pub fn to_text(&self) -> String {
        self.rows.join("\n")
    }

    fn delete_char_by_cursor(&mut self) -> bool {
        if self.cursor.is_zero() {
            return false;
        }

        // Delete linebreak and concat with previous row
        if self.cursor.column == 0 {
            self.cursor.column = self.rows[self.cursor.row - 1].len();
            let text = self.rows.remove(self.cursor.row);
            self.rows[self.cursor.row - 1].push_str(&text);
            self.move_cursor_up();
            return true;
        }

        // Delete normal char
        if self.delete_range_of_row(self.cursor.row, self.cursor.column - 1, self.cursor.column) {
            self.move_cursor_left();
            return true;
        }

        false
    }

    fn delete_range_of_row(&mut self, row_number: usize, start: usize, end: usize) -> bool {
        let left = start.min(end);
        let right = start.max(end).min(self.rows[row_number].len());
        if left == right {
            return false;
        }

        let text = &mut self.rows[row_number];
        let remaining = text.split_off(right);
        text.truncate(left);
        text.push_str(&remaining);

        true
    }

    fn delete_text_by_selection(&mut self) -> bool {
        if self.selection.is_none() {
            return false;
        }

        let start_row = self.find_row_number_of_index(self.selection.start);
        let end_row = self.find_row_number_of_index(self.selection.end);

        if start_row != end_row {
            // Handle last row
            self.delete_range_of_row(
                end_row,
                0,
                // -1 here to manually eliminate the linebreak of last row,
                // which is already removed in the previous step
                self.selection.end - self.find_start_index_of_row(end_row),
            );
            let text = self.rows.remove(end_row);
            self.rows[start_row].push_str(&text);

            // Remove inter rows
            for row_number in start_row + 1..end_row {
                self.rows.remove(row_number);
            }
        }

        // Handle first row
        let (row_start, row_end) = self.find_range_of_row(start_row);
        let (start, end) = self
            .find_selected_range_of_row(start_row)
            .unwrap_or((row_end, row_end + 1));

        let left = start - row_start;
        let mut right = end - row_start;
        if start_row != end_row {
            right -= 1;
        }
        self.delete_range_of_row(start_row, left, right);

        // Reset cursor
        self.cursor.row = start_row;
        self.cursor.column = left;

        self.reset_selection();

        true
    }

    // Row index of glyph index
    // glyph index  0   1   2   3
    //        text  a   b   c   \n
    //         row  0
    // glyph index  4   5   6   7
    //        text  d   e   f   \n
    //         row  1
    fn find_row_number_of_index(&self, index: usize) -> usize {
        let mut left = 0;
        let mut right = self.row_count();
        while left < right {
            let row_number = left + (right - left) / 2;
            let (row_start, row_end) = self.find_range_of_row(row_number);

            if index < row_start {
                right = row_number;
            } else if index >= row_end {
                left = row_number + 1;
            } else {
                return row_number;
            }
        }

        self.row_count() - 1
    }

    // Length of row in glyphs, linebreak included
    fn row_len(&self, row_number: usize) -> usize {
        // +1 to include the linebreak,
        // except for the last row
        let text_len = self.rows[row_number].len();
        if self.is_last_row(row_number) {
            text_len
        } else {
            text_len + 1
        }
    }
}

#[derive(Debug)]
struct TheRowInfo {
    top: usize,
    left: usize,
    bottom: usize,
    right: usize,

    baseline: usize,
    glyph_start: usize,
    glyph_end: usize,
}

struct TheTextRenderer {
    // Dim
    left: usize,
    top: usize,
    width: usize,
    height: usize,

    // Options
    cursor_width: usize,
    cursor_vertical_shrink: usize,
    font_size: f32,

    // State
    actual_size: Vec2<usize>,
    glyphs: Vec<GlyphPosition>,
    row_info: Vec<TheRowInfo>,
    scroll_offset: Vec2<usize>,
}

impl Default for TheTextRenderer {
    fn default() -> Self {
        Self {
            left: 0,
            top: 0,
            width: 0,
            height: 0,

            cursor_width: 2,
            cursor_vertical_shrink: 1,
            font_size: 14.0,

            actual_size: Vec2::zero(),
            glyphs: vec![],
            row_info: vec![],
            scroll_offset: Vec2::zero(),
        }
    }
}

impl TheTextRenderer {
    #[allow(clippy::too_many_arguments)]
    pub fn find_cursor(&self, coord: &Vec2<i32>) -> TheCursor {
        let coord = vec2i(
            coord.x + self.scroll_offset.x.as_i32(),
            coord.y + self.scroll_offset.y.as_i32(),
        );
        let mut cursor = TheCursor::zero();

        if (coord.x < 0 && coord.y < 0) || self.glyphs.is_empty() {
            // Cursor is at the start of all the text
            return cursor;
        }

        for (row_number, row) in self.row_info.iter().enumerate() {
            if coord.y <= row.bottom.as_i32() {
                cursor.row = row_number;

                let start_index = self.row_info[row_number].glyph_start;
                let end_index = self.row_info[row_number].glyph_end;
                cursor.column = end_index - start_index;
                if self.glyphs[end_index].parent != '\n' {
                    cursor.column += 1;
                }

                for i in start_index..=end_index {
                    let glyph = self.glyphs[i];
                    if (glyph.x + glyph.width.as_f32()).as_i32() > coord.x {
                        cursor.column = i - start_index;
                        break;
                    }
                }

                return cursor;
            }
        }

        // Cursor is at the end of all the text
        cursor.row = self.row_count() - 1;
        cursor.column = self.row_info[cursor.row].glyph_end - self.row_info[cursor.row].glyph_start;
        if self.glyphs.last().unwrap().parent != '\n' {
            cursor.column += 1;
        }
        cursor
    }

    pub fn prepare_glyphs(&mut self, text: &str, font: &Font, draw: &TheDraw2D) {
        self.glyphs.clear();
        self.row_info.clear();

        if text.is_empty() {
            panic!("Text is empty");
        };

        let layout = draw.get_text_layout(font, self.font_size, text);
        self.glyphs.clone_from(layout.glyphs());

        self.row_info = layout
            .lines()
            .unwrap()
            .iter()
            .map(|line| {
                let top = (line.baseline_y - line.max_ascent).ceil().as_usize();
                let left = self
                    .glyphs
                    .get(line.glyph_start)
                    .unwrap()
                    .x
                    .ceil()
                    .as_usize();
                let bottom = (line.baseline_y - line.min_descent).ceil().as_usize();
                let right = {
                    let last_glyph = self.glyphs.get(line.glyph_end).unwrap();
                    (last_glyph.x + last_glyph.width.as_f32()).ceil().as_usize()
                };

                self.actual_size.x = self.actual_size.x.max(right);
                self.actual_size.y = self.actual_size.y.max(bottom);

                TheRowInfo {
                    top,
                    left,
                    bottom,
                    right,
                    baseline: line.baseline_y.ceil().as_usize(),
                    glyph_start: line.glyph_start,
                    glyph_end: line.glyph_end,
                }
            })
            .collect();
    }

    pub fn render_cursor(
        &self,
        cursor: &TheCursor,
        cursor_index: usize,
        buffer: &mut TheRGBABuffer,
        style: &mut Box<dyn TheStyle>,
        draw: &TheDraw2D,
    ) {
        let left = self
            .get_text_left(cursor_index)
            .saturating_sub(self.cursor_width / 2);
        let top = self.row_info[cursor.row].baseline - self.cursor_height();
        if self.is_rect_out_of_visible_area(left, top, self.cursor_width, self.cursor_height()) {
            return;
        }

        let left = (self.left + left).saturating_sub(self.scroll_offset.x);
        let top = (self.top + top).saturating_sub(self.scroll_offset.y);

        let stride = buffer.stride();
        draw.rect(
            buffer.pixels_mut(),
            &(left, top, self.cursor_width, self.cursor_height()),
            stride,
            style.theme().color(TextEditCursorColor),
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render_row(
        &self,
        text: &str,
        font: &Font,
        row_number: usize,
        glyph_start_index: usize,
        buffer: &mut TheRGBABuffer,
        style: &mut Box<dyn TheStyle>,
        draw: &TheDraw2D,
    ) {
        let row = &self.row_info[row_number];
        if self.is_rect_out_of_visible_area(
            row.left,
            row.top,
            row.right - row.left,
            row.bottom - row.top,
        ) {
            return;
        }

        // Find the visible text
        let mut visible_text_start_index = 0;
        let mut visible_text_end_index = text.len();
        let mut is_start_index_found = false;
        let mut chars_acc_width = 0;
        for i in 0..text.len() {
            if is_start_index_found && chars_acc_width >= self.scroll_offset.x + self.width {
                visible_text_end_index = i;
                break;
            }
            chars_acc_width = self.get_text_width(glyph_start_index, glyph_start_index + i);
            if !is_start_index_found && chars_acc_width >= self.scroll_offset.x {
                visible_text_start_index = i;
                is_start_index_found = true;
            }
        }
        let visible_text = &text[visible_text_start_index..visible_text_end_index];

        // Render text and clip
        let left = self.left.as_i32() - self.scroll_offset.x.as_i32()
            + self.get_text_left(glyph_start_index + visible_text_start_index).as_i32()
            // Make sure row x start at 0 TODO
            - self.get_text_left(glyph_start_index).as_i32();
        let top = self.top.as_i32() - self.scroll_offset.y.as_i32()
            + self.row_info[row_number].top.as_i32();

        let stride = buffer.stride();
        draw.text_rect_blend_clip(
            buffer.pixels_mut(),
            &vec2i(left, top - 1),
            &(self.left, self.top, self.width, self.height),
            stride,
            font,
            self.font_size,
            visible_text,
            style.theme().color(TextEditTextColor),
            TheHorizontalAlign::Center,
            TheVerticalAlign::Center,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render_selection(
        &self,
        row_number: usize,
        start: usize,
        end: usize,
        buffer: &mut TheRGBABuffer,
        style: &mut Box<dyn TheStyle>,
        draw: &TheDraw2D,
    ) {
        let row = &self.row_info[row_number];
        let row_width = row.right - row.left;
        if self.is_rect_out_of_visible_area(row.left, row.top, row_width, self.row_height()) {
            return;
        }

        let height = self.row_height();
        let width = self.get_text_width(start, end - 1);
        let width = if width == 0 {
            self.linebreak_width()
        } else {
            width
        };

        let left = (self.left + self.get_text_left(start)).saturating_sub(self.scroll_offset.x);
        let top = (self.top + row.top).saturating_sub(self.scroll_offset.y);

        let right = (left + width).min(self.left + self.width);
        let bottom = (top + height).min(self.top + self.height);

        let left = left.max(self.left);
        let top = top.max(self.top);

        let stride = buffer.stride();
        draw.blend_rect(
            buffer.pixels_mut(),
            &(left, top, right - left, bottom - top),
            stride,
            style.theme().color(TextEditSelectionBackground),
        );
    }

    pub fn scroll(&mut self, delta: &Vec2<i32>) -> bool {
        if self.row_info.is_empty() {
            self.scroll_offset = Vec2::zero();
            return true;
        }

        let previous_offset = self.scroll_offset;

        let (start_row, end_row) = self.find_visible_rows();
        let max_width_of_visible_rows = self.row_info[start_row..=end_row]
            .iter()
            .max_by_key(|row| row.right)
            .unwrap()
            .right;

        let rightmost = max_width_of_visible_rows.saturating_sub(self.width);
        self.scroll_offset.x = (self.scroll_offset.x.as_i32() + delta.x)
            .max(0)
            .as_usize()
            .min(rightmost);

        let downmost = self.actual_size.y.saturating_sub(self.height);
        self.scroll_offset.y = (self.scroll_offset.y.as_i32() + delta.y)
            .max(0)
            .as_usize()
            .min(downmost);

        previous_offset != self.scroll_offset
    }

    pub fn scroll_to_cursor(&mut self, cursor_index: usize, cursor_row: usize) {
        let text_width_before_cursor = self.get_text_left(cursor_index);

        // Check if the widget should be scrolled in order to display the cursor
        // Scroll right
        let leftmost = text_width_before_cursor;
        self.scroll_offset.x = self.scroll_offset.x.min(leftmost);
        // Scroll left
        let rightmost = (text_width_before_cursor + self.cursor_width).saturating_sub(self.width);
        self.scroll_offset.x = self.scroll_offset.x.max(rightmost);
        // Scroll down
        let upmost = self.row_info[cursor_row].top;
        self.scroll_offset.y = self.scroll_offset.y.min(upmost);
        // Scroll up
        let downmost = self.row_info[cursor_row].bottom.saturating_sub(self.height);
        self.scroll_offset.y = self.scroll_offset.y.max(downmost);
    }

    pub fn set_dim(&mut self, left: usize, top: usize, width: usize, height: usize) {
        self.left = left;
        self.top = top;
        self.width = width;
        self.height = height;
    }

    fn cursor_height(&self) -> usize {
        self.row_height()
            .saturating_sub(self.cursor_vertical_shrink * 2)
    }

    // Inclusive on both end
    fn find_visible_rows(&self) -> (usize, usize) {
        if self.row_count() == 0 {
            return (0, 0);
        }

        let start_row = self
            .row_info
            .iter()
            .enumerate()
            .find(|(_, row)| row.bottom > self.scroll_offset.y)
            .map(|(idx, _)| idx)
            .unwrap_or_default();
        let end_row = if start_row < self.row_count() - 1 {
            self.row_info[start_row + 1..]
                .iter()
                .enumerate()
                .find(|(_, row)| row.top > self.height + self.scroll_offset.y)
                .map(|(idx, _)| idx)
                .unwrap_or(self.row_count() - 1)
        } else {
            start_row
        };

        (start_row, end_row)
    }

    fn get_text_left(&self, index: usize) -> usize {
        if self.glyphs.is_empty() {
            return 0;
        }

        if let Some(glyph) = self.glyphs.get(index) {
            return glyph.x.ceil().as_usize();
        }

        let last_glyph = self.glyphs[self.glyphs.len() - 1];
        last_glyph.x.ceil().as_usize() + last_glyph.width
    }

    // Support single row only
    // Inclusive on both end
    // Make sure start and end are on the same row
    fn get_text_width(&self, start: usize, end: usize) -> usize {
        if self.glyphs.is_empty() {
            return 0;
        }

        let left = start.min(end);
        let right = start.max(end);
        let last_char_end = self.glyphs[right].x + self.glyphs[right].width.as_f32();
        let right_end = self
            .glyphs
            .get(right + 1)
            .map_or(last_char_end, |next_glyph| {
                if last_char_end < next_glyph.x {
                    next_glyph.x - 1.0
                } else {
                    last_char_end
                }
            });

        (right_end - self.glyphs[left].x).ceil().as_usize()
    }

    fn is_rect_out_of_visible_area(
        &self,
        left: usize,
        top: usize,
        width: usize,
        height: usize,
    ) -> bool {
        top > self.scroll_offset.y + self.height
            || top + height < self.scroll_offset.y
            || left > self.scroll_offset.x + self.width
            || left + width < self.scroll_offset.x
    }

    fn linebreak_width(&self) -> usize {
        (self.font_size * 0.5).ceil().as_usize()
    }

    fn row_count(&self) -> usize {
        self.row_info.len()
    }

    fn row_height(&self) -> usize {
        self.font_size.ceil().as_usize()
    }
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
                if self.range.is_some() && !self.continuous && self.state.to_text() != self.original
                {
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
                if let Ok(value) = self.state.to_text().parse::<i32>() {
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
            TheValue::Int(v) => {
                self.state.set_text(v.to_string());
                self.modified_since_last_tick = true;
                self.is_dirty = true;
            }
            TheValue::Float(v) => {
                self.state.set_text(v.to_string());
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
                    if let Ok(value) = self.state.to_text().parse::<f32>() {
                        let normalized =
                            (value - range_f32.start()) / (range_f32.end() - range_f32.start());
                        pos = Some((normalized * rect.2.as_f32()).as_usize());
                    }
                } else if let Some(range_i32) = range.to_range_i32() {
                    if let Ok(value) = self.state.to_text().parse::<i32>() {
                        let range_diff = range_i32.end() - range_i32.start();
                        let normalized = (value - range_i32.start()) * rect.2.as_i32() / range_diff;
                        pos = Some(normalized.as_usize());
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

        if let Some(font) = &ctx.ui.font {
            shrinker.shrink_by(
                self.padding.0.as_i32(),
                self.padding.1.as_i32(),
                self.padding.2.as_i32(),
                self.padding.3.as_i32(),
            );

            if self.modified_since_last_tick || self.renderer.row_count() == 0 {
                let visible_area = self.dim.to_buffer_shrunk_utuple(&shrinker);
                self.renderer.set_dim(
                    visible_area.0,
                    visible_area.1,
                    visible_area.2,
                    visible_area.3,
                );

                let mut text = self.state.to_text();
                // Indicate a new line, for render and interaction only
                if text.ends_with('\n') || text.is_empty() {
                    text.push('\n');
                }
                self.renderer.prepare_glyphs(&text, font, &ctx.draw);
                self.renderer
                    .scroll_to_cursor(self.state.find_cursor_index(), self.state.cursor.row);
            }

            for i in 0..self.state.row_count() {
                if let Some((start, end)) = self.state.find_selected_range_of_row(i) {
                    self.renderer
                        .render_selection(i, start, end, buffer, style, &ctx.draw);
                }

                self.renderer.render_row(
                    &self.state.rows[i],
                    font,
                    i,
                    self.state.find_start_index_of_row(i),
                    buffer,
                    style,
                    &ctx.draw,
                );
            }

            if ctx.ui.has_focus(self.id()) {
                self.renderer.render_cursor(
                    &self.state.cursor,
                    self.state.find_cursor_index(),
                    buffer,
                    style,
                    &ctx.draw,
                );
            }
        }

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
        self.state.set_text(text);
        self.modified_since_last_tick = true;
        self.is_dirty = true;
    }
    fn set_font_size(&mut self, font_size: f32) {
        self.renderer.font_size = font_size;
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
