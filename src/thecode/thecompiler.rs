use crate::prelude::*;

use super::theexecution::TheExePipeline;

pub struct TheCompiler {

}

impl Default for TheCompiler {
    fn default() -> Self {
        TheCompiler::new()
    }
}

impl TheCompiler {
    pub fn new() -> Self {
        Self {
        }
    }

    pub fn compile(&mut self, ctx: TheCodeContext) -> Result<TheExePipeline, String>{

        let mut pipe = TheExePipeline::new();

        Ok(pipe)
    }
}
