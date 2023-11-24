use theframework::*;

pub mod browser;
pub mod editor;
pub mod sidebar;

use crate::editor::CodeEditor;

pub mod prelude {
    pub use crate::sidebar::*;
    pub use theframework::prelude::*;
}

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");

    let demo = CodeEditor::new();
    let mut app = TheApp::new();

    _ = app.run(Box::new(demo));
}
