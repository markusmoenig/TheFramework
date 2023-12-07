//use crate::prelude::*;

pub mod theatom;
pub mod thecodeeditor;
pub mod thecodegrid;
pub mod thecodeobject;
pub mod thecompiler;
pub mod theexecution;

pub mod prelude {
    pub use crate::thecode::theatom::{TheAtom, TheAtomKind};
    pub use crate::thecode::thecodeeditor::TheCodeEditor;
    pub use crate::thecode::thecodegrid::TheCodeGrid;
    pub use crate::thecode::thecodeobject::TheCodeObject;
    pub use crate::thecode::thecompiler::{TheCompiler, TheCompilerContext, TheCompilerError};
    pub use crate::thecode::theexecution::{TheExeNode, TheExeNodeCall, TheExePipeline, TheExeEnvironment};
}

pub struct TheReturnCode {}
//    fn new() -> Self
//}
