use crate::prelude::*;
use crate::Embedded;
use fontdue::Font;

use std::sync::mpsc::Sender;

pub struct TheUIContext {
    pub font: Option<Font>,
    pub code_font: Option<Font>,
    icons: FxHashMap<String, TheRGBABuffer>,

    pub focus: Option<TheId>,
    pub keyboard_focus: Option<TheId>,
    pub hover: Option<TheId>,
    pub overlay: Option<TheId>,

    pub state_events_sender: Option<Sender<TheEvent>>,

    pub redraw_all: bool,
    pub relayout: bool,
}

impl Default for TheUIContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TheUIContext {
    pub fn new() -> Self {
        let mut font: Option<Font> = None;
        let mut code_font: Option<Font> = None;
        let mut icons: FxHashMap<String, TheRGBABuffer> = FxHashMap::default();

        for file in Embedded::iter() {
            let name = file.as_ref();
            if name.starts_with("fonts/Roboto-Bold") {
                if let Some(font_bytes) = Embedded::get(name) {
                    if let Ok(f) =
                        Font::from_bytes(font_bytes.data, fontdue::FontSettings::default())
                    {
                        font = Some(f);
                    }
                }
            } else if name.starts_with("fonts/Source") {
                if let Some(font_bytes) = Embedded::get(name) {
                    if let Ok(f) =
                        Font::from_bytes(font_bytes.data, fontdue::FontSettings::default())
                    {
                        code_font = Some(f);
                    }
                }
            } else if name.starts_with("icons/") {
                if let Some(file) = Embedded::get(name) {
                    let data = std::io::Cursor::new(file.data);

                    let decoder = png::Decoder::new(data);
                    if let Ok(mut reader) = decoder.read_info() {
                        let mut buf = vec![0; reader.output_buffer_size()];
                        let info = reader.next_frame(&mut buf).unwrap();
                        let bytes = &buf[..info.buffer_size()];

                        let mut cut_name = name.replace("icons/", "");
                        cut_name = cut_name.replace(".png", "");
                        icons.insert(
                            cut_name.to_string(),
                            TheRGBABuffer::from(bytes.to_vec(), info.width, info.height),
                        );
                    }
                }
            }
        }

        Self {
            focus: None,
            keyboard_focus: None,
            hover: None,
            overlay: None,

            font,
            code_font,
            icons,

            state_events_sender: None,

            redraw_all: false,
            relayout: false,
        }
    }

    /// Returns an icon of the given name from the embedded style icons
    pub fn icon(&self, name: &str) -> Option<&TheRGBABuffer> {
        if let Some(icon) = self.icons.get(&name.to_string()) {
            return Some(icon);
        }
        None
    }

    /// Sets the focus to the given widget
    pub fn set_focus(&mut self, id: &TheId) {
        if !id.equals(&self.focus) {
            if let Some(focus) = &self.focus {
                self.send(TheEvent::LostFocus(focus.clone()));
            }
            self.send(TheEvent::GainedFocus(id.clone()));
            self.focus = Some(id.clone());
        }
    }

    /// Clears the focus state.
    pub fn clear_focus(&mut self) {
        self.focus = None;
    }

    /// Checks if the given id has focus
    pub fn has_focus(&self, id: &TheId) -> bool {
        id.equals(&self.focus)
    }

    /// Sets the hover to the given widget
    pub fn set_hover(&mut self, id: &TheId) {
        if !id.equals(&self.hover) {
            if let Some(hover) = &self.hover {
                self.send(TheEvent::LostHover(hover.clone()));
            }
            self.send(TheEvent::GainedHover(id.clone()));
            self.hover = Some(id.clone());
        }
    }

    /// Clears the hover state.
    pub fn clear_hover(&mut self) {
        self.hover = None;
    }

    /// Sets the overlay to the given widget. This will call the draw_overlay method of the widget after all other draw calls (for menus etc).
    pub fn set_overlay(&mut self, id: &TheId) {
        self.overlay = Some(id.clone());
    }

    /// Clears
    pub fn clear_overlay(&mut self) {
        self.overlay = None;
        self.redraw_all = true;
    }

    /// Indicates that the state of the given widget changed
    pub fn send_widget_state_changed(&mut self, id: &TheId, state: TheWidgetState) {
        self.send(TheEvent::StateChanged(id.clone(), state));
    }

    pub fn set_widget_state(&mut self, name: String, state: TheWidgetState) {
        self.send(TheEvent::SetState(name, state));
    }

    /// Sends the given state event
    pub fn send(&mut self, event: TheEvent) {
        if let Some(sender) = &mut self.state_events_sender {
            sender.send(event).unwrap();
        }
    }

    /// Indicates that the state of the given widget changed
    pub fn send_widget_value_changed(&mut self, id: &TheId, value: TheValue) {
        self.send(TheEvent::ValueChanged(id.clone(), value));
    }
}
