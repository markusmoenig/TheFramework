//use crate::prelude::*;

pub mod theatom;
pub mod thecodegrid;
pub mod thecompiler;
pub mod theexecution;
pub mod thecodeeditor;

pub mod prelude {
    pub use crate::thecode::theatom::{TheAtom, TheAtomKind};
    pub use crate::thecode::thecodegrid::TheCodeGrid;
    pub use crate::thecode::thecompiler::{TheCompiler, TheCompilerContext, TheCompilerError};
    pub use crate::thecode::theexecution::{TheExeNode, TheExeNodeCall, TheExePipeline};
    pub use crate::thecode::thecodeeditor::TheCodeEditor;
}

pub struct TheReturnCode {}
//    fn new() -> Self
//}
