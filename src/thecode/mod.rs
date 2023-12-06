use crate::prelude::*;

pub mod theatom;
pub mod thecodegrid;
pub mod thecompiler;
pub mod theexecution;

pub mod prelude {
    pub use crate::thecode::theatom::{TheAtom, TheAtomKind};
    pub use crate::thecode::thecodegrid::TheCodeGrid;
    pub use crate::thecode::thecompiler::{TheCompiler, TheCompilerContext};
    pub use crate::thecode::theexecution::{TheExeNode, TheExeNodeCall, TheExePipeline};
}

pub struct TheReturnCode {}
//    fn new() -> Self
//}
