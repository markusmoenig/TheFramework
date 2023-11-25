use crate::prelude::*;

pub mod theatom;
pub mod thecodecontext;
pub mod thecompiler;
pub mod theexecution;

pub mod prelude {
    pub use crate::thecode::theatom::{TheAtom, TheAtomKind};
    pub use crate::thecode::thecodecontext::TheCodeContext;
    pub use crate::thecode::thecompiler::TheCompiler;
    pub use crate::thecode::theexecution::{TheExeNode, TheExeNodeCall, TheExePipeline};
}

pub struct TheReturnCode {}
//    fn new() -> Self
//}
