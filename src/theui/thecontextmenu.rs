use crate::prelude::*;

// Item

pub struct TheContextMenuItem {
    pub name: String,
    pub event: Option<TheEvent>,
    pub layout: Option<Box<dyn TheLayout>>,
}

impl TheContextMenuItem {
    pub fn event(name: String, event: Option<TheEvent>) -> Self {
        Self {
            name,
            event,
            layout: None,
        }
    }

    pub fn layout(name: String, layout: Option<Box<dyn TheLayout>>) -> Self {
        Self {
            name,
            event: None,
            layout,
        }
    }
}

// Menu

pub struct TheContextMenu {
    pub items: Vec<TheContextMenuItem>,
}

impl Default for TheContextMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl TheContextMenu {
    pub fn new() -> Self {
        Self { items: vec![] }
    }

    /// Add an item,
    pub fn add(&mut self, item: TheContextMenuItem) {
        self.items.push(item);
    }
}
