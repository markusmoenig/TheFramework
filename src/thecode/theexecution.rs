use crate::prelude::*;

pub type TheExeNodeCall =
    fn(ops: Vec<TheValue>, values: Vec<TheValue>) -> TheValue;

pub struct TheExeNode {
    pub call: TheExeNodeCall,
    pub values: Vec<TheValue>
}

impl TheExeNode {
    pub fn new(call: TheExeNodeCall, values: Vec<TheValue>) -> Self {
        Self {
            call,
            values
        }
    }
}

pub struct TheExePipeline {
    pub nodes: Vec<TheExeNode>
}

impl Default for TheExePipeline {
    fn default() -> Self {
        TheExePipeline::new()
    }
}

impl TheExePipeline {
    pub fn new() -> Self {
        Self {
            nodes: vec![]
        }
    }
}