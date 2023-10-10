use crate::prelude::*;
use crate::Embedded;
use fontdue::Font;

pub struct TheUIContext {

    pub font: Option<Font>,
    pub code_font: Option<Font>,

    pub focus: Option<Uuid>,

}

impl TheUIContext {
    pub fn new() -> Self {

        let mut font : Option<Font> = None;
        let mut code_font : Option<Font> = None;
        let mut icons : FxHashMap<String, (Vec<u8>, u32, u32)> = FxHashMap::default();

        for file in Embedded::iter() {
            let name = file.as_ref();
            println!("{:?}", name);
            if name.starts_with("fonts/Roboto") {
                if let Some(font_bytes) = Embedded::get(name) {
                    if let Some(f) = Font::from_bytes(font_bytes.data, fontdue::FontSettings::default()).ok() {
                        font = Some(f);
                    }
                }
            } else
            if name.starts_with("fonts/Source") {
                if let Some(font_bytes) = Embedded::get(name) {
                    if let Some(f) = Font::from_bytes(font_bytes.data, fontdue::FontSettings::default()).ok() {
                        code_font = Some(f);
                    }
                }
            } else
            if name.starts_with("icons/") {
                if let Some(file) = Embedded::get(name) {
                    let data = std::io::Cursor::new(file.data);

                    let decoder = png::Decoder::new(data);
                    if let Ok(mut reader) = decoder.read_info() {
                        let mut buf = vec![0; reader.output_buffer_size()];
                        let info = reader.next_frame(&mut buf).unwrap();
                        let bytes = &buf[..info.buffer_size()];

                        let mut cut_name = name.replace("icons/", "");
                        cut_name = cut_name.replace(".png", "");
                        icons.insert(cut_name.to_string(), (bytes.to_vec(), info.width, info.height));
                    }
                }
            }
        }

        Self {
            focus: None,

            font: font,
            code_font: code_font
        }
    }
}
