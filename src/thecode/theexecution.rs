use crate::prelude::*;

pub type TheExeNodeCall = fn(stack: &mut Vec<TheValue>, values: &Vec<TheValue>);

#[derive(Clone)]
pub struct TheExeNode {
    pub call: TheExeNodeCall,
    pub values: Vec<TheValue>,
}

impl TheExeNode {
    pub fn new(call: TheExeNodeCall, values: Vec<TheValue>) -> Self {
        Self { call, values }
    }
}

#[derive(Clone)]
pub struct TheExePipeline {
    pub nodes: Vec<TheExeNode>,
}

impl Default for TheExePipeline {
    fn default() -> Self {
        TheExePipeline::new()
    }
}

impl TheExePipeline {
    pub fn new() -> Self {
        Self { nodes: vec![] }
    }

    pub fn add(&mut self, node: TheExeNode) {
        self.nodes.push(node);
    }

    pub fn execute(&mut self) {
        let mut stack: Vec<TheValue> = Vec::with_capacity(10);

        for n in &self.nodes {
            (n.call)(&mut stack, &n.values);
            println!("{:?}", stack);
        }
    }
}
