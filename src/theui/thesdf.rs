use crate::prelude::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum TheSDF {
    Circle(Vec2f, f32),
}

use TheSDF::*;

impl TheSDF {
    pub fn distance(&self, p: Vec2f) -> f32 {
        match self {
            Circle(pos, radius) => length(p - pos) - radius,
        }
    }

    // pub fn to_vec2f(&self) -> Option<Vec2f> {
    //     match self {
    //         Float2(v) => Some(*v),
    //         _ => None,
    //     }
    // }

    /// Returns a description of the SDF as string.
    pub fn describe(&self) -> String {
        match self {
            Circle(pos, radius) => format!("Circle: {:?} {}", pos, radius),
        }
    }
}
