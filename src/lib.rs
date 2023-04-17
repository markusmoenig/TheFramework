pub mod theapp;
pub mod thetrait;
pub mod thecontext;
pub mod thedraw2d;

pub use crate::theapp::TheApp as TheApp;
pub use crate::thetrait::TheTrait as TheTrait;
pub use crate::thecontext::TheContext as TheContext;

pub enum WidgetKey {
    Escape,
    Return,
    Delete,
    Up,
    Right,
    Down,
    Left,
    Space,
    Tab
}

pub mod prelude {
    pub use crate::theapp::TheApp;
    pub use crate::thetrait::TheTrait;
    pub use crate::thecontext::TheContext;
    pub use crate::thedraw2d::TheDraw2D;
    pub use crate::WidgetKey;
}