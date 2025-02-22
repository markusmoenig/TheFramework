use theframework::*;

pub mod editor;
pub mod project;

use crate::editor::CodeEditor;

pub mod prelude {
    pub use theframework::prelude::*;

    pub use crate::project::Project;
    pub use ::serde::{Deserialize, Serialize};
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    std::env::set_var("RUST_BACKTRACE", "1");

    let code: CodeEditor = CodeEditor::new();
    let mut app = TheApp::new();

    () = app.run(Box::new(code));
}
