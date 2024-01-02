use crate::prelude::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TheCodeSandbox {

    /// The id of the sandbox.
    pub id: Uuid,

    /// The modules with callable codegrid functions. These make up the behavior of an entity.
    #[serde(skip)]
    pub modules: FxHashMap<Uuid, TheCodeModule>,
    /// The global external functions added by the host.
    #[serde(skip)]
    pub globals: FxHashMap<String, TheCodeNode>,

    /// The objects with values. These make up the state of an entity.
    pub objects: FxHashMap<Uuid, TheCodeObject>,

    /// Debug switch.
    pub debug_mode: bool,

    // Runtimes
    /// Redirects object aliases (like Self, Target etc.) to a given Uuid.
    #[serde(skip)]
    pub aliases: FxHashMap<String, Uuid>,

    /// Function return value.
    #[serde(skip)]
    pub func_rc: Option<TheValue>,

    /// The call stack of modules.
    #[serde(skip)]
    pub module_stack: Vec<Uuid>,

    /// The call stack of the codegrid source of the module.
    #[serde(skip)]
    pub codegrid_stack: Vec<Uuid>,

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

            id: Uuid::new_v4(),

            objects: FxHashMap::default(),
            modules: FxHashMap::default(),
            globals: FxHashMap::default(),

            debug_mode: false,

            aliases: FxHashMap::default(),

            func_rc: None,
            module_stack: vec![],
            call_stack: vec![],
            codegrid_stack: vec![],
            debug_modules: FxHashMap::default(),
        }
    }

    /// Clear the runtime states.
    pub fn clear_runtime_states(&mut self) {
        self.aliases = FxHashMap::default();
        self.func_rc = None;
        self.module_stack = vec![];
        self.call_stack = vec![];
        self.codegrid_stack = vec![];
        self.debug_modules = FxHashMap::default();
    }

    /// Adds a globlal function to the environment.
    pub fn add_global(&mut self, name: &str, node: TheCodeNode) {
        self.globals.insert(name.to_string(), node);
    }

    /// Insert a module into the environment.
    pub fn insert_module(&mut self, module: TheCodeModule) {
        self.modules.insert(module.id, module);
    }

    /// Add an object into the sandbox.
    pub fn add_object(&mut self, object: TheCodeObject) {
        self.objects.insert(object.id, object);
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
    pub fn call_global(&mut self, location: (u16, u16), stack: &mut Vec<TheValue>, name: &String) {
        // Temporarily remove the node from the map
        if let Some(mut node) = self.globals.remove(name) {
            // Call the function with a mutable reference to node.data
            node.data.location = location;
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

    /// Returns a reference to the aliased object.
    pub fn get_object(&self, name: &String) -> Option<&TheCodeObject> {
        if let Some(id) = self.aliases.get(name) {
            if let Some(object) = self.objects.get(id) {
                return Some(object);
            }
        }
        None
    }

    /// Returns a mutable reference to the aliased object.
    pub fn get_object_mut(&mut self, name: &String) -> Option<&mut TheCodeObject> {
        if let Some(id) = self.aliases.get(name) {
            if let Some(object) = self.objects.get_mut(id) {
                return Some(object);
            }
        }
        None
    }

    /// Returns a mutable reference to the current object with an alias of "self".
    pub fn get_self_mut(&mut self) -> Option<&mut TheCodeObject> {
        if let Some(id) = self.aliases.get("self") {
            if let Some(object) = self.objects.get_mut(id) {
                return Some(object);
            }
        }
        None
    }

    /// Pushes the current module to the module stack.
    pub fn push_current_module(&mut self, module_id: Uuid, codegrid_id: Uuid) {
        self.module_stack.push(module_id);
        self.codegrid_stack.push(codegrid_id);
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

    /// Returns the debug values for a given entity id.
    pub fn get_codegrid_debug_module(&self, entity_id: Uuid) -> TheDebugModule {
        for (index, id) in self.codegrid_stack.iter().enumerate() {
            if *id == entity_id {
                if let Some(module_id) = self.module_stack.get(index) {
                    if let Some(dv) = self.debug_modules.get(module_id) {
                        return dv.clone();
                    }
                }
            }
        }
        TheDebugModule::new()
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
