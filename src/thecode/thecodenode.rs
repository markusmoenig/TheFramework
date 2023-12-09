use crate::prelude::*;

pub type TheCodeNodeCall =
    fn(stack: &mut Vec<TheValue>, values: &Vec<TheValue>, env: &mut TheCodeSandbox);

#[derive(Clone, Debug)]
pub struct TheCodeNode {
    pub call: TheCodeNodeCall,
    pub values: Vec<TheValue>,
}

impl TheCodeNode {
    pub fn new(call: TheCodeNodeCall, values: Vec<TheValue>) -> Self {
        Self { call, values }
    }
}
