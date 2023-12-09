use crate::prelude::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum TheCodeGridMessageType {
    Error,
    Value,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TheCodeGridMessage {
    pub message_type: TheCodeGridMessageType,
    pub message: String,
}

impl TheCodeGridMessage {
    pub fn new(message_type: TheCodeGridMessageType, message: String) -> Self {
        Self {
            message_type,
            message,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TheCodeGrid {
    pub code: FxHashMap<(u32, u32), TheAtom>,
    pub messages: FxHashMap<(u32, u32), TheCodeGridMessage>,
    pub current_pos: Option<(u32, u32)>,
    pub max_pos: Option<(u32, u32)>,
}

impl Default for TheCodeGrid {
    fn default() -> Self {
        TheCodeGrid::new()
    }
}

impl TheCodeGrid {
    pub fn new() -> Self {
        Self {
            code: FxHashMap::default(),
            messages: FxHashMap::default(),
            current_pos: None,
            max_pos: None,
        }
    }

    /// Returns the max xy values in the grid
    pub fn max_xy(&self) -> Option<(u32, u32)> {
        let mut max_x = None;
        let mut max_y = None;

        for (x, y) in self.code.keys() {
            max_x = Some(max_x.map_or(*x, |mx| std::cmp::max(mx, *x)));
            max_y = Some(max_y.map_or(*y, |my| std::cmp::max(my, *y)));
        }

        match (max_x, max_y) {
            (Some(max_x), Some(max_y)) => Some((max_x, max_y)),
            _ => None, // Return None if the grid is empty
        }
    }

    /// Returns the next TheAtom in the grid.
    pub fn get_next(&mut self, peek: bool) -> TheAtom {
        if let Some(max_pos) = self.max_xy() {
            if let Some((mut x, mut y)) = self.current_pos {
                // Check if we're at or beyond the maximum position
                if x == max_pos.0 && y == max_pos.1 {
                    return TheAtom::End; // Reached the end of the grid
                }

                // Attempt to find the next non-empty position
                loop {
                    if x == max_pos.0 {
                        x = 0;
                        y += 1;
                    } else {
                        x += 1;
                    }

                    if let Some(atom) = self.code.get(&(x, y)) {
                        if !peek {
                            self.current_pos = Some((x, y));
                        }
                        return atom.clone(); // Found a non-empty position
                    }

                    if x == max_pos.0 && y == max_pos.1 {
                        return TheAtom::End; // Reached the end of the grid
                    }
                }
            } else {
                // Start from the first position if current_pos is None
                if let Some(atom) = self.code.get(&(0, 0)) {
                    if !peek {
                        self.current_pos = Some((0, 0));
                    }
                    return atom.clone();
                }
            }
        }

        TheAtom::End
    }

    /// Reset the grid iterator.
    pub fn reset_iterator(&mut self) {
        self.current_pos = None;
    }

    /// Clears the messages for the grid.
    pub fn clear_messages(&mut self) {
        self.messages = FxHashMap::default();
    }

    /// Adds a message to the grid.
    pub fn add_message(&mut self, location: (u32, u32), message: TheCodeGridMessage) {
        self.messages.insert(location, message);
    }

    /// Returns the message for the given location (if any).
    pub fn message(&self, location: (u32, u32)) -> Option<TheCodeGridMessage> {
        let mut message: Option<TheCodeGridMessage> = None;
        if let Some(m) = self.messages.get(&location) {
            message = Some(m.clone());
        }
        message
    }
}
