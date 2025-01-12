use crate::prelude::*;
use indexmap::IndexMap;
use std::ops::RangeInclusive;

/// The items that can be added to TheNodeUI
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum TheNodeUIItem {
    /// Text: Id, Name, Status, Value, DefaultValue, Continuous
    Text(String, String, String, String, Option<String>, bool),
    /// Selector: Id, Name, Status, Values, Value
    Selector(String, String, String, Vec<String>, i32),
    /// Float Edit Slider: Id, Name, Status, Value, Range, Continuous
    FloatEditSlider(String, String, String, f32, RangeInclusive<f32>, bool),
    /// Float Slider: Id, Name, Status, Value, Range, DefaultValue, Continuous
    FloatSlider(String, String, String, f32, RangeInclusive<f32>, f32, bool),
    /// Int Edit Slider: Id, Name, Status, Value, Range, Continuous
    IntEditSlider(String, String, String, i32, RangeInclusive<i32>, bool),
    /// Int Slider: Id, Name, Status, Value, Range, DefaultValue, Continuous
    IntSlider(String, String, String, i32, RangeInclusive<i32>, i32, bool),
}

impl TheNodeUIItem {
    /// Returns the `id` for the item
    pub fn id(&self) -> &str {
        match self {
            TheNodeUIItem::Text(id, _, _, _, _, _) => id,
            TheNodeUIItem::Selector(id, _, _, _, _) => id,
            TheNodeUIItem::FloatEditSlider(id, _, _, _, _, _) => id,
            TheNodeUIItem::FloatSlider(id, _, _, _, _, _, _) => id,
            TheNodeUIItem::IntEditSlider(id, _, _, _, _, _) => id,
            TheNodeUIItem::IntSlider(id, _, _, _, _, _, _) => id,
        }
    }
}

use TheNodeUIItem::*;

/// A container for UI items. Supports adding them to a text layout or handling events for updating the values.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TheNodeUI {
    items: IndexMap<String, TheNodeUIItem>,
}

impl Default for TheNodeUI {
    fn default() -> Self {
        Self::new()
    }
}

impl TheNodeUI {
    pub fn new() -> Self {
        Self {
            items: IndexMap::new(),
        }
    }

    /// Adds a new item to the UI
    pub fn add_item(&mut self, item: TheNodeUIItem) -> Option<TheNodeUIItem> {
        self.items.insert(item.id().into(), item)
    }

    /// Removes an item by its ID
    pub fn remove_item(&mut self, id: &str) -> Option<TheNodeUIItem> {
        self.items.shift_remove(id)
    }

    /// Retrieves a reference to an item by its ID
    pub fn get_item(&self, id: &str) -> Option<&TheNodeUIItem> {
        self.items.get(id)
    }

    /// Retrieves a mutable reference to an item by its ID
    pub fn get_item_mut(&mut self, id: &str) -> Option<&mut TheNodeUIItem> {
        self.items.get_mut(id)
    }

    /// Lists all items in the UI
    pub fn list_items(&self) -> impl Iterator<Item = (&String, &TheNodeUIItem)> {
        self.items.iter()
    }

    /// Add the items to the given text layout.
    pub fn apply_to_text_layout(&self, layout: &mut dyn TheTextLayoutTrait) {
        layout.clear();
        for (_, item) in &self.items {
            match item {
                Text(id, name, status, value, default_value, continous) => {
                    let mut edit = TheTextLineEdit::new(TheId::named(id));
                    edit.set_text(value.clone());
                    edit.set_continuous(*continous);
                    edit.set_status_text(status);
                    edit.set_info_text(default_value.clone());
                    layout.add_pair(name.clone(), Box::new(edit));
                }
                Selector(id, name, status, values, value) => {
                    let mut dropdown = TheDropdownMenu::new(TheId::named(id));
                    for item in values {
                        dropdown.add_option(item.clone());
                    }
                    dropdown.set_selected_index(*value);
                    dropdown.set_status_text(status);
                    layout.add_pair(name.clone(), Box::new(dropdown));
                }
                FloatEditSlider(id, name, status, value, range, continous) => {
                    let mut slider = TheTextLineEdit::new(TheId::named(id));
                    slider.set_value(TheValue::Float(*value));
                    slider.set_range(TheValue::RangeF32(range.clone()));
                    slider.set_continuous(*continous);
                    slider.set_status_text(status);
                    layout.add_pair(name.clone(), Box::new(slider));
                }
                FloatSlider(id, name, status, value, range, default_value, continous) => {
                    let mut slider = TheSlider::new(TheId::named(id));
                    slider.set_value(TheValue::Float(*value));
                    slider.set_default_value(TheValue::Float(*default_value));
                    slider.set_range(TheValue::RangeF32(range.clone()));
                    slider.set_continuous(*continous);
                    slider.set_status_text(status);
                    layout.add_pair(name.clone(), Box::new(slider));
                }
                IntEditSlider(id, name, status, value, range, continous) => {
                    let mut slider = TheTextLineEdit::new(TheId::named(id));
                    slider.set_value(TheValue::Int(*value));
                    slider.set_range(TheValue::RangeI32(range.clone()));
                    slider.set_continuous(*continous);
                    slider.set_status_text(status);
                    layout.add_pair(name.clone(), Box::new(slider));
                }
                IntSlider(id, name, status, value, range, default_value, continous) => {
                    let mut slider = TheSlider::new(TheId::named(id));
                    slider.set_value(TheValue::Int(*value));
                    slider.set_default_value(TheValue::Int(*default_value));
                    slider.set_range(TheValue::RangeI32(range.clone()));
                    slider.set_continuous(*continous);
                    slider.set_status_text(status);
                    layout.add_pair(name.clone(), Box::new(slider));
                }
            }
        }
    }

    /// Handle an event and update the item values if necessary
    pub fn handle_event(&mut self, event: TheEvent) -> bool {
        let mut updated = false;
        #[allow(clippy::single_match)]
        match event {
            TheEvent::ValueChanged(id, event_value) => {
                if let Some(item) = self.get_item_mut(&id.name) {
                    match item {
                        Text(_, _, _, value, _, _) => {
                            if let TheValue::Text(v) = event_value {
                                *value = v;
                                updated = true;
                            }
                        }
                        Selector(_, _, _, _, value) => {
                            if let TheValue::Int(v) = event_value {
                                *value = v;
                                updated = true;
                            }
                        }
                        FloatEditSlider(_, _, _, value, _, _) => {
                            if let TheValue::Float(v) = event_value {
                                *value = v;
                                updated = true;
                            }
                        }
                        FloatSlider(_, _, _, value, _, _, _) => {
                            if let TheValue::Float(v) = event_value {
                                *value = v;
                                updated = true;
                            }
                        }
                        IntEditSlider(_, _, _, value, _, _) => {
                            if let TheValue::Int(v) = event_value {
                                *value = v;
                                updated = true;
                            }
                        }
                        IntSlider(_, _, _, value, _, _, _) => {
                            if let TheValue::Int(v) = event_value {
                                *value = v;
                                updated = true;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        updated
    }

    // pub fn create_canvas(&self) -> TheCanvas {
    //     let mut canvas = TheCanvas::default();

    //     canvas
    // }
}
