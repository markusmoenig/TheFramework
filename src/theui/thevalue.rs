use crate::prelude::*;

#[derive(Clone, Debug)]
pub enum TheValue {
    Coordinate(Vec2i),
    Text(String),
    Char(char),
    KeyCode(TheKeyCode),
}

use TheValue::*;

impl TheValue {
    pub fn to_vec2i(&self) -> Option<Vec2i> {
        match self {
            Coordinate(v) => Some(*v),
            _ => None,
        }
    }

    pub fn to_string(&self) -> Option<String> {
        match self {
            Text(v) => Some(v.clone()),
            _ => None,
        }
    }

    pub fn to_char(&self) -> Option<char> {
        match self {
            Char(v) => Some(*v),
            _ => None,
        }
    }

    pub fn to_key_code(&self) -> Option<TheKeyCode> {
        match self {
            KeyCode(v) => Some(v.clone()),
            _ => None,
        }
    }
}
