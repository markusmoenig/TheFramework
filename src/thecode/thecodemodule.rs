use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct TheCodeModule {
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
            functions: FxHashMap::default(),
        }
    }

    /// Insert a function into the module.
    pub fn insert_function(&mut self, name: String, function: TheCodeFunction) {
        self.functions.insert(name, function);
    }

    /// Execute the module by calling the main function.
    pub fn execute(&mut self, sandbox: &mut TheCodeSandbox) {
        if let Some(main) = self.functions.get_mut(&"main".to_string()) {
            main.execute(sandbox);
        }
    }
}