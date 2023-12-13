use crate::prelude::*;
use std::ops::RangeInclusive;

/// TheValue contains all possible values used by widgets and layouts. Encapsulating them in an enum alllows easy transfer and comparison of values.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum TheValue {
    Empty,
    Coordinate(Vec2i),
    Float(f32),
    Int(i32),
    Text(String),
    Char(char),
    KeyCode(TheKeyCode),
    RangeI32(RangeInclusive<i32>),
    RangeF32(RangeInclusive<f32>),
    #[cfg(feature = "code")]
    CodeObject(TheCodeObject),
}

use TheValue::*;

impl TheValue {
    pub fn to_vec2i(&self) -> Option<Vec2i> {
        match self {
            Coordinate(v) => Some(*v),
            _ => None,
        }
    }

    pub fn to_i32(&self) -> Option<i32> {
        match self {
            Int(v) => Some(*v),
            Text(t) => t.parse::<i32>().ok(),
            _ => None,
        }
    }

    pub fn to_f32(&self) -> Option<f32> {
        match self {
            Float(v) => Some(*v),
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

    pub fn to_range_i32(&self) -> Option<RangeInclusive<i32>> {
        match self {
            RangeI32(v) => Some(v.clone()),
            _ => None,
        }
    }

    pub fn to_range_f32(&self) -> Option<RangeInclusive<f32>> {
        match self {
            RangeF32(v) => Some(v.clone()),
            _ => None,
        }
    }

    /// Returns a description of the value as string.
    pub fn describe(&self) -> String {
        match self {
            Empty => "Empty".to_string(),
            Coordinate(v) => format!("Coordinate: {:?}", v),
            Float(f) => f.to_string(),
            Int(i) => i.to_string(),
            Text(s) => s.clone(),
            Char(c) => c.to_string(),
            CodeObject(_) => "CodeObject".to_string(),
            KeyCode(k) => format!("KeyCode: {:?}", k),
            RangeI32(r) => format!("RangeI32: {:?}", r),
            RangeF32(r) => format!("RangeF32: {:?}", r),
        }
    }
}
