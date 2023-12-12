use crate::prelude::*;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct TheDrop {
    pub id: TheId,
    pub data: String,
    pub title: String,
    pub image: TheRGBABuffer,

    pub position: Option<Vec2i>,
    pub offset: Vec2i,
}

impl TheDrop {
    pub fn new(id: TheId) -> Self {
        Self {
            id,
            data: String::new(),
            title: String::new(),
            image: TheRGBABuffer::empty(),
            position: None,
            offset: Vec2i::zero(),
        }
    }

    pub fn set_position(&mut self, position: Vec2i) {
        self.position = Some(position);
    }

    pub fn set_offset(&mut self, offset: Vec2i) {
        self.offset = offset;
    }

    pub fn set_data(&mut self, json: String) {
        self.data = json;
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    pub fn set_image(&mut self, image: TheRGBABuffer) {
        self.image = image;
    }
}
