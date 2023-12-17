use crate::prelude::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Project {
    pub name: String,
    pub bundle: TheCodeBundle,

    #[serde(skip)]
    pub undo_stack: TheUndoStack,
}

impl Default for Project {
    fn default() -> Self {
        Self::new()
    }
}

impl Project {
    pub fn new() -> Self {
        let mut bundle = TheCodeBundle::default();
        bundle.insert_grid(TheCodeGrid::default());

        Self {
            name: "Untitled".to_string(),
            bundle,
            undo_stack: TheUndoStack::default(),
        }
    }
}
