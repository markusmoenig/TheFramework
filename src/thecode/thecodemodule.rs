use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct TheCodeModule {
    pub name: String,
    pub uuid: Uuid,
    pub functions: FxHashMap<String, TheCodeFunction>,
}

impl Default for TheCodeModule {
    fn default() -> Self {
        TheCodeModule::new()
    }
}

impl TheCodeModule {
    pub fn new() -> Self {
        Self {
            name: "Unnamed".to_string(),
            uuid: Uuid::new_v4(),
            functions: FxHashMap::default(),
        }
    }

    /// Insert a function into the module.
    pub fn insert_function(&mut self, name: String, function: TheCodeFunction) {
        self.functions.insert(name, function);
    }

    /// Get a function from the module.
    pub fn get_function(&self, name: &String) -> Option<&TheCodeFunction> {
        self.functions.get(name)
    }

    /// Get a mutable function from the module.
    pub fn get_function_mut(&mut self, name: &String) -> Option<&mut TheCodeFunction> {
        self.functions.get_mut(name)
    }

    /// Execute the module by calling the main function.
    pub fn execute(&mut self, sandbox: &mut TheCodeSandbox) {
        if let Some(main) = self.functions.get_mut(&"main".to_string()) {
            let clone = main.clone();

            sandbox.push_current_module(self.uuid);
            sandbox.call_stack.push(clone);

            main.execute(sandbox);
            sandbox.call_stack.pop();
        }
    }
}
