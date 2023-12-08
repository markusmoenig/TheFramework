use crate::prelude::*;

pub type TheExeNodeCall = fn(stack: &mut Vec<TheValue>, values: &Vec<TheValue>, env: &mut TheExeEnvironment);

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

    pub fn execute(&mut self, env: &mut TheExeEnvironment) {
        let mut stack: Vec<TheValue> = Vec::with_capacity(10);

        for n in &self.nodes {
            (n.call)(&mut stack, &n.values, env);
            println!("{:?}", stack);
        }

        println!("{:?}", env.local);
    }
}

pub type TheGetVarCall = fn(object_name: String, var_name: String) -> Option<TheValue>;
pub type TheSetVarCall = fn(object_name: String, var_name: String, value: TheValue);
pub type TheFnCall = fn(args: Vec<TheValue>) -> Option<TheValue>;

#[derive(Clone)]
pub struct TheExeEnvironment {
    /// The function used to retrieve an object value.
    pub get_var: Option<TheGetVarCall>,
    /// The function used to set an object value.
    pub set_var: Option<TheSetVarCall>,
    /// The function calls to native Rust functions for this environment.
    pub functions: FxHashMap<String, TheFnCall>,

    /// The local variables
    pub local: Vec<TheCodeObject>,
}

impl Default for TheExeEnvironment {
    fn default() -> Self {
        TheExeEnvironment::new()
    }
}

impl TheExeEnvironment {
    pub fn new() -> Self {
        Self {
            get_var: None,
            set_var: None,
            functions: FxHashMap::default(),
            local: vec![TheCodeObject::default()]
        }
    }

    /// Insert a function into the environment.
    pub fn insert(&mut self, name: String, function: TheFnCall) {
        self.functions.insert(name, function);
    }

    /// Returns the given local variable by reversing the local stack.
    pub fn get_local(&self, name: &String) -> Option<&TheValue> {
        for local in self.local.iter().rev() {
            if let Some(var) = local.get(name) {
                return Some(var);
            }
        }
        None
    }
}
