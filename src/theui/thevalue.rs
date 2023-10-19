use crate::prelude::*;

#[derive(Clone, Debug)]
pub enum TheValue {
    Coordinate(Vec2i),
    Text(String)
}

use TheValue::*;

impl TheValue {

    pub fn to_vec2i(&self) -> Option<Vec2i> {
        match self {
            Coordinate(v) => {
                Some(*v)
            },
            _ => {
                None
            }
        }
    }

    pub fn to_string(&self) -> Option<String> {
        match self {
            Text(v) => {
                Some(v.clone())
            },
            _ => {
                None
            }
        }
    }
}