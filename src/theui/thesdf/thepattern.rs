use crate::prelude::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ThePattern {
    Solid(TheColor),
}

use ThePattern::*;

impl ThePattern {
    pub fn get_color(&self, _p: Vec2f, _distance: &f32, highlight: Option<&TheColor>) -> TheColor {
        match self {
            Solid(color) => {
                if let Some(highlight) = highlight {
                    highlight.clone()
                } else {
                    color.clone()
                }
            }
        }
    }
}
