//pub mod render;
pub mod thechunk;
pub mod node;
pub mod thecompiler;
pub mod thescanner;
pub mod value;
pub mod thevm;

pub mod prelude {
    pub use crate::thescript::thechunk::*;
    pub use crate::thescript::thecompiler::*;
    pub use crate::thescript::thescanner::*;
    pub use crate::thescript::value::*;
    pub use crate::thescript::thevm::*;
    pub use crate::thescript::node::*;
}

// pub use crate::vm::VM as VM;
// pub use crate::vm::InterpretError as InterpretError;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
