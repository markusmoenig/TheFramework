use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct TheCodeFunction {
    pub name: String,
    pub nodes: Vec<TheCodeNode>,
    pub local: Vec<TheCodeObject>,

    pub public: bool,
    pub arguments: Vec<String>,
}

impl Default for TheCodeFunction {
    fn default() -> Self {
        TheCodeFunction::new()
    }
}

impl TheCodeFunction {
    pub fn new() -> Self {
        Self {
            name: "main".to_string(),
            local: vec![TheCodeObject::default()],
            nodes: vec![],
            public: false,
            arguments: vec![],
        }
    }

    pub fn named(name: String) -> Self {
        Self {
            name,
            local: vec![TheCodeObject::default()],
            nodes: vec![],
            public: false,
            arguments: vec![],
        }
    }

    /// Add a node.
    pub fn add_node(&mut self, node: TheCodeNode) {
        self.nodes.push(node);
    }

    /// Returns true if the function is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
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

    /// Execute the function
    pub fn execute(&mut self, sandbox: &mut TheCodeSandbox) {
        let mut stack: Vec<TheValue> = Vec::with_capacity(10);

        for n in &self.nodes {
            (n.call)(&mut stack, &n.values, sandbox);
            println!("{:?}", stack);
        }
    }
}
