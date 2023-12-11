use crate::prelude::*;

/// Represents a single undo/redo operation.
///
/// It stores the type of the operation along with the data required to undo or redo the action.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct TheUndo {
    // Type of the undo operation.
    pub undo_type: String,

    // Data required to perform the undo operation.
    pub undo_data: String,
    // Data required to perform the redo operation.
    pub redo_data: String,
}

impl TheUndo {
    /// Creates a new `TheUndo` instance.
    ///
    /// # Arguments
    ///
    /// * `undo_type` - A string slice that holds the type of the undo operation.
    ///
    /// # Returns
    ///
    /// A new instance of `TheUndo`.
    pub fn new(undo_type: &str) -> Self {
        Self {
            undo_type: undo_type.to_string(),
            undo_data: String::new(),
            redo_data: String::new(),
        }
    }

    /// Sets the undo data.
    ///
    /// # Arguments
    ///
    /// * `json` - A string containing the undo data in JSON format.
    pub fn set_undo_data(&mut self, json: String) {
        self.undo_data = json;
    }

    /// Sets the redo data.
    ///
    /// # Arguments
    ///
    /// * `json` - A string containing the redo data in JSON format.
    pub fn set_redo_data(&mut self, json: String) {
        self.redo_data = json;
    }
}

/// Represents a stack of undo/redo operations.
///
/// This struct manages a stack of `TheUndo` instances, allowing for undo and redo functionality.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct TheUndoStack {
    // Stack of undo/redo operations.
    pub stack: Vec<TheUndo>,

    // Current index in the undo stack.
    pub index: isize,
}

impl Default for TheUndoStack {
    /// Creates a new `TheUndoStack` instance with default values.
    ///
    /// # Returns
    ///
    /// A new instance of `TheUndoStack` with empty stack and index set to -1.
    fn default() -> Self {
        Self::new()
    }
}

impl TheUndoStack {
    /// Creates a new `TheUndoStack` instance.
    ///
    /// # Returns
    ///
    /// A new instance of `TheUndoStack`.
    pub fn new() -> Self {
        Self {
            stack: vec![],
            index: -1,
        }
    }

    /// Clears the undo stack.
    ///
    /// This method resets the stack and index to their default state.
    pub fn clear(&mut self) {
        self.stack = vec![];
        self.index = -1;
    }

    /// Checks if an undo operation is possible.
    ///
    /// # Returns
    ///
    /// `true` if an undo operation can be performed, `false` otherwise.
    pub fn has_undo(&self) -> bool {
        self.index >= 0
    }

    /// Checks if a redo operation is possible.
    ///
    /// # Returns
    ///
    /// `true` if a redo operation can be performed, `false` otherwise.
    pub fn has_redo(&self) -> bool {
        if self.index >= -1 && self.index < self.stack.len() as isize - 1 {
            return true;
        }
        false
    }

    /// Adds a new `TheUndo` instance to the stack.
    ///
    /// # Arguments
    ///
    /// * `undo` - An instance of `TheUndo` to be added to the stack.
    pub fn add(&mut self, undo: TheUndo) {
        let to_remove = self.stack.len() as isize - self.index - 1;
        for _i in 0..to_remove {
            self.stack.pop();
        }
        self.stack.push(undo);
        self.index += 1;
    }

    /// Performs an undo operation and returns the relevant data.
    ///
    /// # Returns
    ///
    /// A tuple containing the undo type and undo data.
    pub fn undo(&mut self) -> (String, String) {
        let rc = (
            self.stack[self.index as usize].undo_type.clone(),
            self.stack[self.index as usize].undo_data.clone(),
        );
        self.index -= 1;
        rc
    }

    /// Performs a redo operation and returns the relevant data.
    ///
    /// # Returns
    ///
    /// A tuple containing the redo type and redo data.
    pub fn redo(&mut self) -> (String, String) {
        self.index += 1;
        (
            self.stack[self.index as usize].undo_type.clone(),
            self.stack[self.index as usize].redo_data.clone(),
        )
    }
}
