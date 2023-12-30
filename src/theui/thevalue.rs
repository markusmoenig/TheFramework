use crate::prelude::*;
use std::ops::RangeInclusive;

/// TheValue contains all possible values used by widgets and layouts. Encapsulating them in an enum alllows easy transfer and comparison of values.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum TheValue {
    Empty,
    Bool(bool),
    Float(f32),
    Int(i32),
    Text(String),
    Char(char),
    Int2(Vec2i),
    Float2(Vec2f),
    Int3(Vec3i),
    Float3(Vec3f),
    Int4(Vec4i),
    Float4(Vec4f),
    Position(Vec3f),
    Tile(String, Uuid),
    KeyCode(TheKeyCode),
    RangeI32(RangeInclusive<i32>),
    RangeF32(RangeInclusive<f32>),
    ColorObject(TheColor),
    #[cfg(feature = "code")]
    CodeObject(TheCodeObject),
}

use TheValue::*;

impl TheValue {
    pub fn to_vec2i(&self) -> Option<Vec2i> {
        match self {
            Int2(v) => Some(*v),
            _ => None,
        }
    }

    pub fn to_vec2f(&self) -> Option<Vec2f> {
        match self {
            Float2(v) => Some(*v),
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
            Tile(name, _id) => Some(name.clone()),
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

    pub fn to_color(&self) -> Option<TheColor> {
        match self {
            ColorObject(v) => Some(v.clone()),
            _ => None,
        }
    }

    /// Add a value to another value. Returns None if the values are not compatible.
    pub fn add(&self, other: &TheValue) -> Option<TheValue> {
        if let TheValue::Int(a) = self {
            match other {
                TheValue::Int(b) => Some(TheValue::Int(a + b)),
                TheValue::Float(b) => Some(TheValue::Float(*a as f32 + b)),
                _ => None,
            }
        } else if let TheValue::Float(a) = self {
            match other {
                TheValue::Int(b) => Some(TheValue::Float(a + *b as f32)),
                TheValue::Float(b) => Some(TheValue::Float(a + b)),
                _ => None,
            }
        } else if let TheValue::Position(a) = self {
            match other {
                TheValue::Int(b) => Some(TheValue::Position(vec3f(a.x + *b as f32, a.y, a.z))),
                TheValue::Int2(b) => Some(TheValue::Position(vec3f(
                    a.x + b.x as f32,
                    a.y + b.y as f32,
                    a.z,
                ))),
                TheValue::Int3(b) => Some(TheValue::Position(vec3f(
                    a.x + b.x as f32,
                    a.y + b.y as f32,
                    a.z + b.z as f32,
                ))),
                TheValue::Float(b) => Some(TheValue::Position(vec3f(a.x + *b, a.y, a.z))),
                TheValue::Float2(b) => Some(TheValue::Position(vec3f(a.x + b.x, a.y + b.y, a.z))),
                TheValue::Float3(b) => {
                    Some(TheValue::Position(vec3f(a.x + b.x, a.y + b.y, a.z + b.z)))
                }
                _ => None,
            }
        } else {
            None
        }
    }

    /// Returns a description of the value as string.
    pub fn to_kind(&self) -> String {
        match self {
            Empty => "Empty".to_string(),
            Bool(_v) => "Bool".to_string(),
            Float(_v) => "Float".to_string(),
            Int(_i) => "Integer".to_string(),
            Text(_s) => "Text".to_string(),
            Int2(v) => format!("Int2: {:?}", v),
            Float2(v) => format!("Float22: {:?}", v),
            Int3(v) => format!("Int3: {:?}", v),
            Float3(v) => format!("Float3: {:?}", v),
            Int4(v) => format!("Int4: {:?}", v),
            Float4(v) => format!("Float4: {:?}", v),
            Position(v) => format!("Position: {:?}", v),
            Tile(_v, _id) => "Tile".to_string(),
            Char(c) => c.to_string(),
            #[cfg(feature = "code")]
            CodeObject(_) => "Object".to_string(),
            KeyCode(k) => format!("KeyCode: {:?}", k),
            RangeI32(r) => format!("RangeI32: {:?}", r),
            RangeF32(r) => format!("RangeF32: {:?}", r),
            ColorObject(c) => format!("Color: {:?}", c),
        }
    }

    /// Returns a description of the value as string.
    pub fn describe(&self) -> String {
        match self {
            Empty => "Empty".to_string(),
            Bool(v) => {
                if *v {
                    "True".to_string()
                } else {
                    "False".to_string()
                }
            }
            Float(v) => {
                if v.fract() == 0.0 {
                    format!("{:.1}", *v)
                } else {
                    v.to_string()
                }
            }
            Int(i) => i.to_string(),
            Text(s) => s.clone(),
            Int2(v) => format!("({}, {})", v.x, v.y),
            Float2(v) => format!("({}, {})", v.x, v.y),
            Int3(v) => format!("({}, {}, {})", v.x, v.y, v.z),
            Float3(v) => format!("({}, {}, {})", v.x, v.y, v.z),
            Int4(v) => format!("({}, {}, {}, {})", v.x, v.y, v.z, v.w),
            Float4(v) => format!("({}, {}, {}, {})", v.x, v.y, v.z, v.w),
            Position(v) => format!("({}, {})", v.x, v.y),
            Tile(name, _id) => name.clone(),
            Char(c) => c.to_string(),
            #[cfg(feature = "code")]
            CodeObject(_) => "Object".to_string(),
            KeyCode(k) => format!("KeyCode: {:?}", k),
            RangeI32(r) => format!("RangeI32: {:?}", r),
            RangeF32(r) => format!("RangeF32: {:?}", r),
            ColorObject(c) => format!("Color: {:?}", c),
        }
    }
}
