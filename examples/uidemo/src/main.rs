use theframework::*;

pub mod demo;
use crate::demo::UIDemo;

pub mod prelude {
    pub use theframework::prelude::*;
}

fn main() {
    let demo = UIDemo::new();
    let mut app = TheApp::new();

    _ = app.run(Box::new(demo));
}
