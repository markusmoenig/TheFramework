pub mod theapp;
pub mod thecontext;
pub mod thedraw2d;
pub mod thetrait;

#[cfg(feature = "ui")]
pub mod theui;

pub use crate::theapp::TheApp;
pub use crate::thecontext::TheContext;
pub use crate::thetrait::TheTrait;

#[cfg(feature = "ui")]
pub use crate::theui::TheUI;

#[derive(Clone)]
pub enum WidgetKey {
    Escape,
    Return,
    Delete,
    Up,
    Right,
    Down,
    Left,
    Space,
    Tab,
}

pub mod prelude {

    pub use maths_rs::prelude::*;

    pub use crate::theapp::TheApp;
    pub use crate::thecontext::TheContext;
    pub use crate::thedraw2d::TheDraw2D;
    pub use crate::thetrait::TheTrait;
    pub use crate::WidgetKey;

    #[cfg(feature = "renderer")]
    pub use therenderer::prelude::*;

    #[cfg(feature = "ui")]
    pub use crate::theui::prelude::*;
}
