use crate::prelude::*;

pub type TheCodeNodeCall =
    fn(stack: &mut Vec<TheValue>, values: &TheCodeNodeData, sandbox: &mut TheCodeSandbox);

#[derive(Clone, Debug)]
pub struct TheCodeNodeData {
    pub values: Vec<TheValue>,
    pub sub_calls: Vec<TheCodeNodeCall>,
    pub location: (u16, u16),
}

impl TheCodeNodeData {
    pub fn location(location: (u16, u16)) -> Self {
        Self {
            values: vec![],
            sub_calls: vec![],
            location,
        }
    }

    pub fn location_values(location: (u16, u16), values: Vec<TheValue>) -> Self {
        Self {
            values,
            sub_calls: vec![],
            location,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TheCodeNode {
    pub call: TheCodeNodeCall,
    pub data: TheCodeNodeData,
}

impl TheCodeNode {
    pub fn new(call: TheCodeNodeCall, data: TheCodeNodeData) -> Self {
        Self { call, data }
    }
}
