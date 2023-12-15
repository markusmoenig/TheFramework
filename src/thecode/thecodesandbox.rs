use crate::prelude::*;

pub type TheGetVarCall = fn(object_name: String, var_name: String) -> Option<TheValue>;
pub type TheSetVarCall = fn(object_name: String, var_name: String, value: TheValue);
pub type TheFnCall = fn(args: Vec<TheValue>) -> Option<TheValue>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TheCodeSandbox {
    /// The function used to retrieve an object value.
    #[serde(skip)]
    pub get_var: Option<TheGetVarCall>,
    /// The function used to set an object value.
    #[serde(skip)]
    pub set_var: Option<TheSetVarCall>,
    /// The modules with callable codegrid functions.
    #[serde(skip)]
    pub modules: FxHashMap<Uuid, TheCodeModule>,
    /// The global external functions added by the host.
    #[serde(skip)]
    pub globals: FxHashMap<String, TheCodeNode>,

    pub objects: FxHashMap<String, TheCodeObject>,

    pub debug_mode: bool,

    // Runtimes
    /// Function return value.
    #[serde(skip)]
    pub func_rc: Option<TheValue>,

    /// The call stack of modules.
    #[serde(skip)]
    pub module_stack: Vec<Uuid>,

    /// The call stack of functions.
    #[serde(skip)]
    pub call_stack: Vec<TheCodeFunction>,

    /// The call stack of functions.
    #[serde(skip)]
    pub debug_modules: FxHashMap<Uuid, TheDebugModule>,
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
            modules: FxHashMap::default(),
            globals: FxHashMap::default(),

            debug_mode: false,

            func_rc: None,
            module_stack: vec![],
            call_stack: vec![],
            debug_modules: FxHashMap::default(),
        }
    }

    /// Clear the runtime states.
    pub fn clear_runtime_states(&mut self) {
        self.func_rc = None;
        self.module_stack = vec![];
        self.call_stack = vec![];
        self.debug_modules = FxHashMap::default();
    }

    /// Adds a globlal function to the environment.
    pub fn add_global(&mut self, name: &str, node: TheCodeNode) {
        self.globals.insert(name.to_string(), node);
    }

    /// Insert a module into the environment.
    pub fn insert_module(&mut self, module: TheCodeModule) {
        self.modules.insert(module.uuid, module);
    }

    /// Insert an object into the environment.
    pub fn insert_object(&mut self, name: String, function: TheCodeObject) {
        self.objects.insert(name, function);
    }

    /// Get a clone of the function from the environment.
    pub fn get_function_cloned(&self, module_id: Uuid, name: &String) -> Option<TheCodeFunction> {
        if let Some(module) = self.modules.get(&module_id) {
            if let Some(function) = module.get_function(name) {
                return Some(function.clone());
            }
        }
        None
    }

    // /// Call a global, external function provided by the host.
    pub fn call_global(&mut self, stack: &mut Vec<TheValue>, name: &String) {
        // Temporarily remove the node from the map
        if let Some(mut node) = self.globals.remove(name) {
            // Call the function with a mutable reference to node.data
            (node.call)(stack, &mut node.data, self);

            // Reinsert the node back into the map
            self.globals.insert(name.clone(), node);
        }
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

    /// Pushes the current module to the module stack.
    pub fn push_current_module(&mut self, module_id: Uuid) {
        self.module_stack.push(module_id);
        self.debug_modules.entry(module_id).or_default();
    }

    /// Sets a debug value in the current module.
    pub fn set_debug_value(&mut self, location: (u16, u16), value: TheValue) {
        if let Some(module_id) = self.module_stack.last() {
            if let Some(debug_module) = self.debug_modules.get_mut(module_id) {
                debug_module.values.insert(location, value);
            }
        }
    }

    /// Returns the debug values for a given module id.
    pub fn get_module_debug_module(&self, module_id: Uuid) -> TheDebugModule {
        if let Some(dv) = self.debug_modules.get(&module_id) {
            dv.clone()
        } else {
            TheDebugModule::new()
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TheDebugModule {
    pub values: FxHashMap<(u16, u16), TheValue>,
    pub executed: FxHashSet<(u16, u16)>,
}

impl Default for TheDebugModule {
    fn default() -> Self {
        TheDebugModule::new()
    }
}

impl TheDebugModule {
    pub fn new() -> Self {
        Self {
            values: FxHashMap::default(),
            executed: FxHashSet::default(),
        }
    }
}
