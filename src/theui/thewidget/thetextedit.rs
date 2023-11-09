use crate::prelude::*;
use fontdue::{Font, Metrics};

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum TheTextEditMode {
    Text,
    Rhai,
    Settings,
}

pub struct TheTextEdit {
    id: TheId,
    limiter: TheSizeLimiter,

    text: String,
    original: String,
    position: usize,

    font_size: f32,

    dim: TheDim,
    is_dirty: bool,

    cursor_offset: usize,
    pub cursor_pos: (usize, usize),
    pub cursor_rect: (usize, usize, usize, usize),

    needs_update: bool,
    pub mode: TheTextEditMode,

    line_numbers_buffer: Vec<u8>,
    line_numbers_size: (usize, usize),

    text_buffer: Vec<u8>,
    text_buffer_size: (usize, usize),

    metrics: FxHashMap<char, (Metrics, Vec<u8>)>,
    advance_width: usize,
    advance_height: usize,

    shift: bool,
    ctrl: bool,
    alt: bool,
    logo: bool,

    //pub theme: Theme,
    //pub settings: Settings,
    error: Option<(String, Option<usize>)>,

    mouse_wheel_delta: (isize, isize),
    offset: (isize, isize),
    max_offset: (usize, usize),

    range_buffer: (usize, usize),
    range_start: Option<(usize, usize)>,
    range_end: Option<(usize, usize)>,

    last_pos: (usize, usize),
    last_click: u128,
    click_stage: i32,

    code_safe_rect: (usize, usize, usize, usize),

    pub drag_pos: Option<(usize, usize)>,
}

impl TheWidget for TheTextEdit {
    fn new(id: TheId) -> Self
    where
        Self: Sized,
    {
        let limiter = TheSizeLimiter::new();

        Self {
            id,
            limiter,

            text: "".to_string(),
            original: "".to_string(),
            position: 0,

            font_size: 14.5,

            dim: TheDim::zero(),
            is_dirty: false,

            cursor_offset: 0,
            cursor_pos: (0, 0),
            cursor_rect: (0, 0, 2, 0),

            needs_update: true,
            mode: TheTextEditMode::Rhai,

            line_numbers_buffer: vec![0; 1],
            line_numbers_size: (0, 0),

            text_buffer: vec![0; 1],
            text_buffer_size: (0, 0),

            metrics: FxHashMap::default(),
            advance_width: 10,
            advance_height: 22,

            shift: false,
            ctrl: false,
            alt: false,
            logo: false,

            //theme                       : Theme::new(),
            //settings                    : Settings::new(),
            error: None,

            mouse_wheel_delta: (0, 0),
            offset: (0, 0),
            max_offset: (0, 0),

            range_buffer: (0, 0),
            range_start: None,
            range_end: None,

            last_pos: (0, 0),
            last_click: 0,
            click_stage: 0,

            code_safe_rect: (0, 0, 0, 0),

            drag_pos: None,
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

    #[allow(clippy::single_match)]
    fn on_event(&mut self, event: &TheEvent, ctx: &mut TheContext) -> bool {
        let mut redraw = false;
        //println!("event ({}): {:?}", self.widget_id.name, event);
        match event {
            TheEvent::MouseDown(coord) => {
                ctx.ui.set_focus(self.id());
                self.is_dirty = true;
                redraw = true;
                self.original = self.text.clone();

                self.position = 0;
                if let Some(coord) = coord.to_vec2i() {
                    let x = coord.x - 7;
                    let mut offset = 0;
                    let mut found = false;
                    if !self.text.is_empty() && x >= 0 {
                        for i in 1..self.text.len() {
                            let txt = &self.text[0..i];
                            if let Some(font) = &ctx.ui.font {
                                let size = ctx.draw.get_text_size(font, self.font_size, txt);
                                if size.0 as i32 >= x {
                                    offset = i;
                                    found = true;
                                    break;
                                }
                            }
                        }
                        if found {
                            self.position = offset;
                        } else {
                            self.position = self.text.len();
                        }
                    }
                }
            }
            TheEvent::MouseDragged(coord) => {
                self.is_dirty = true;
                redraw = true;

                self.position = 0;
                if let Some(coord) = coord.to_vec2i() {
                    let x = coord.x - 7;
                    let mut offset = 0;
                    let mut found = false;
                    if !self.text.is_empty() && x >= 0 {
                        for i in 1..self.text.len() {
                            let txt = &self.text[0..i];
                            if let Some(font) = &ctx.ui.font {
                                let size = ctx.draw.get_text_size(font, self.font_size, txt);
                                if size.0 as i32 >= x {
                                    offset = i;
                                    found = true;
                                    break;
                                }
                            }
                        }
                        if found {
                            self.position = offset;
                        } else {
                            self.position = self.text.len();
                        }
                    }
                }
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
                        ctx.ui.send_widget_value_changed(
                            self.id(),
                            TheValue::Text(self.text.clone()),
                        );
                    }
                }
            }
            TheEvent::LostFocus(_id) => {
                if self.text != self.original {
                    ctx.ui
                        .send_widget_value_changed(self.id(), TheValue::Text(self.text.clone()));
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

        style.draw_text_edit_border(buffer, self, &mut shrinker, ctx);

        ctx.draw.rect(
            buffer.pixels_mut(),
            &self.dim.to_buffer_shrunk_utuple(&shrinker),
            stride,
            style.theme().color(TextEditBackground),
        );

        shrinker.shrink_by(5, 0, 5, 0);

        if let Some(font) = &ctx.ui.font {
            if !self.text.is_empty() {
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
}

pub trait TheTextEditTrait: TheWidget {
    fn set_text(&mut self, text: String);
}

impl TheTextEditTrait for TheTextEdit {
    fn set_text(&mut self, text: String) {
        self.text = text;
        self.position = 0;
    }
}
