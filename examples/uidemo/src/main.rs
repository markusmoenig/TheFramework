use theframework::*;

pub mod demo;
pub mod sidebar;
pub mod browser;

use crate::demo::UIDemo;

pub mod prelude {
    pub use crate::sidebar::*;
    pub use theframework::prelude::*;
}

fn main() {
    // std::env::set_var("RUST_BACKTRACE", "1");

    let demo = UIDemo::new();
    let mut app = TheApp::new();

    _ = app.run(Box::new(demo));
}
