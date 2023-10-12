use crate::prelude::*;

pub mod colorbutton;

pub mod prelude {
    pub use crate::theui::thewidget::colorbutton::TheColorButton;
    pub use crate::theui::thewidget::TheWidget;
    pub use crate::theui::thewidget::TheWidgetId;
    pub use crate::theui::thewidget::TheWidgetState;
}

/// TheWidget trait defines the asbtract functionality of a widget.
#[allow(unused)]
pub trait TheWidget {
    fn new(name: String) -> Self
    where
        Self: Sized;

    fn id(&self) -> &TheWidgetId;
    fn state(&self) -> &TheWidgetState;

    fn init(&mut self, ctx: &mut TheContext) {}

    /// Returns a reference to the dimensions of the widget.
    fn dim(&self) -> &TheDim;

    /// Returns a mutable reference to the dimensions of the widget.
    fn dim_mut(&mut self) -> &mut TheDim;

    /// Set the dimensions of the widget
    fn set_dim(&mut self, dim: TheDim) {}

    /// Draw the widget in the given style
    fn draw(
        &mut self,
        buffer: &mut TheRGBABuffer,
        style: &mut Box<dyn TheStyle>,
        ctx: &mut TheContext,
    ) {
    }

    fn update(&mut self, ctx: &mut TheContext) {}

    fn needs_update(&mut self, ctx: &mut TheContext) -> bool {
        false
    }

    fn on_event(&mut self, event: &TheEvent, ctx: &mut TheContext) {}
}

/// Defines the identifier for a widget, its name and Uuid.
#[derive(Clone, Debug)]
pub struct TheWidgetId {
    pub name: String,
    pub uuid: Uuid,
}

impl TheWidgetId {
    pub fn new(name: String) -> Self {
        Self {
            name,
            uuid: Uuid::new_v4(),
        }
    }

    /// Matches the id against optional names and uuids.
    pub fn matches(&self, name: Option<&String>, uuid: Option<&Uuid>) -> bool {
        if name.is_none() && uuid.is_none() {
            return false;
        }

        name == Some(&self.name) || uuid == Some(&self.uuid)
    }

    /// Checks if the ids are equal (reference the same widget).
    pub fn equals(&self, other: &Option<TheWidgetId>) -> bool {
        if let Some(other) = other {
            if self.uuid == other.uuid {
                return true;
            }
        }
        false
    }
}

/// Defines the widget state.
#[derive(Clone, Debug)]
pub struct TheWidgetState {
    pub mouse_over: bool,
    pub mouse_clicked: bool,
    pub toggle_state: bool,
}

impl Default for TheWidgetState {
    fn default() -> Self {
        Self::new()
    }
}

impl TheWidgetState {
    pub fn new() -> Self {
        Self {
            mouse_over: false,
            mouse_clicked: false,
            toggle_state: false,
        }
    }
}
