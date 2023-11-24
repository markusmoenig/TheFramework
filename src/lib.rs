pub mod theapp;
pub mod thecolor;
pub mod thecontext;
pub mod thedraw2d;
pub mod thetrait;

#[cfg(feature = "ui")]
pub mod theui;

#[cfg(feature = "code")]
pub mod thecode;

pub mod thescript;

pub use crate::theapp::TheApp;
pub use crate::thecontext::TheContext;
pub use crate::thetrait::TheTrait;

#[cfg(feature = "ui")]
pub use crate::theui::TheUI;

use rust_embed::RustEmbed;
#[derive(RustEmbed)]
#[folder = "embedded/"]
#[exclude = "*.txt"]
#[exclude = "*.DS_Store"]
pub struct Embedded;
pub use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum TheKeyCode {
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
    pub use rustc_hash::*;
    pub use uuid::*;

    pub use crate::theapp::TheApp;
    pub use crate::thecolor::TheColor;
    pub use crate::thecontext::TheContext;
    pub use crate::thedraw2d::TheDraw2D;
    pub use crate::thedraw2d::TheHorizontalAlign;
    pub use crate::thedraw2d::TheVerticalAlign;

    pub use crate::thetrait::TheTrait;
    pub use crate::TheKeyCode;

    #[cfg(feature = "renderer")]
    pub use therenderer::prelude::*;

    #[cfg(feature = "ui")]
    pub use crate::theui::prelude::*;

    #[cfg(feature = "code")]
    pub use crate::thecode::prelude::*;

    pub use crate::thescript::prelude::*;
}
