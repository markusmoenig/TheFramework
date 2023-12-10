use crate::prelude::*;

pub type TheGetVarCall = fn(object_name: String, var_name: String) -> Option<TheValue>;
pub type TheSetVarCall = fn(object_name: String, var_name: String, value: TheValue);
pub type TheFnCall = fn(args: Vec<TheValue>) -> Option<TheValue>;

#[derive(Serialize, Deserialize, Clone)]
pub struct TheCodeSandbox {
    /// The function used to retrieve an object value.
    #[serde(skip)]
    pub get_var: Option<TheGetVarCall>,
    /// The function used to set an object value.
    #[serde(skip)]
    pub set_var: Option<TheSetVarCall>,
    /// The function calls to native Rust functions for this environment.
    #[serde(skip)]
    pub functions: FxHashMap<String, TheCodeFunction>,

    pub objects: FxHashMap<String, TheCodeObject>,

    /// The local variables
    #[serde(skip)]
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
            objects: FxHashMap::default(),
            functions: FxHashMap::default(),
            call_stack: vec![TheCodeFunction::default()],
        }
    }

    /// Insert a function into the environment.
    pub fn insert_function(&mut self, name: String, function: TheCodeFunction) {
        self.functions.insert(name, function);
    }

    /// Insert an object into the environment.
    pub fn insert_object(&mut self, name: String, function: TheCodeObject) {
        self.objects.insert(name, function);
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
