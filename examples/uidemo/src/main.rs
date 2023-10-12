use theframework::*;

pub mod demo;
pub mod sidebar;

use crate::demo::UIDemo;

pub mod prelude {
    pub use theframework::prelude::*;
    pub use crate::sidebar::*;
}

fn main() {
    let demo = UIDemo::new();
    let mut app = TheApp::new();

    _ = app.run(Box::new(demo));
}
