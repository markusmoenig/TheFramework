use crate::prelude::*;
use crate::Embedded;
use fontdue::Font;

use std::sync::mpsc::Sender;

pub struct TheUIContext {
    pub font: Option<Font>,
    pub code_font: Option<Font>,

    pub focus: Option<TheWidgetId>,

    pub state_events_sender: Option<Sender<TheEvent>>,
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
        let mut icons: FxHashMap<String, (Vec<u8>, u32, u32)> = FxHashMap::default();

        for file in Embedded::iter() {
            let name = file.as_ref();
            // println!("{:?}", name);
            if name.starts_with("fonts/Roboto") {
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
                            (bytes.to_vec(), info.width, info.height),
                        );
                    }
                }
            }
        }

        Self {
            focus: None,

            font,
            code_font,

            state_events_sender: None,
        }
    }

    /// Sets the focus to the given widget
    pub fn set_focus(&mut self, id: &TheWidgetId) {
        if !id.equals(&self.focus) {
            self.send_state(TheEvent::Focus(id.clone()));
            self.focus = Some(id.clone());
        }
    }

    /// Sends the given state event
    fn send_state(&mut self, event: TheEvent) {
        if let Some(sender) = &mut self.state_events_sender {
            sender.send(event).unwrap();
        }
    }
}
