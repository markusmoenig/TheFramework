use crate::prelude::*;

pub type TheGetVarCall = fn(object_name: String, var_name: String) -> Option<TheValue>;
pub type TheSetVarCall = fn(object_name: String, var_name: String, value: TheValue);
pub type TheFnCall = fn(args: Vec<TheValue>) -> Option<TheValue>;

#[derive(Clone)]
pub struct TheCodeSandbox {
    /// The function used to retrieve an object value.
    pub get_var: Option<TheGetVarCall>,
    /// The function used to set an object value.
    pub set_var: Option<TheSetVarCall>,
    /// The function calls to native Rust functions for this environment.
    pub functions: FxHashMap<String, TheFnCall>,

    /// The local variables
    pub call_stack: Vec<TheCodeFunction>,
}

impl Default for TheCodeSandbox {
    fn default() -> Self {
        TheCodeSandbox::new()
    }
}

impl TheCodeSandbox {
    pub fn new() -> Self {
        Self {
            get_var: None,
            set_var: None,
            functions: FxHashMap::default(),
            call_stack: vec![TheCodeFunction::default()],
        }
    }

    /// Insert a function into the environment.
    pub fn insert(&mut self, name: String, function: TheFnCall) {
        self.functions.insert(name, function);
    }

    /// Returns the given local variable by reversing the local stack.
    pub fn get_local(&self, name: &String) -> Option<&TheValue> {
        if let Some(function) = self.call_stack.last() {
            if let Some(var) = function.get_local(name) {
                return Some(var);
            }
        }
        None
    }
}
