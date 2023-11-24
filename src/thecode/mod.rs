use crate::prelude::*;

pub mod thecompiler;
pub mod theatom;
pub mod thecodecontext;
pub mod theexecution;

pub mod prelude {
    pub use crate::thecode::theatom::TheAtom;
    pub use crate::thecode::thecodecontext::TheCodeContext;
    pub use crate::thecode::theexecution::TheExeNode;
}

pub struct TheReturnCode {


}
//    fn new() -> Self
//}