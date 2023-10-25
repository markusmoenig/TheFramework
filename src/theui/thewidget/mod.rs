use crate::prelude::*;

pub mod thecolorbutton;
pub mod thedropdownmenu;
pub mod themenubar;
pub mod thesectionbarbutton;
pub mod thesectionbar;
pub mod theswitchbar;
pub mod thetext;
pub mod thetextlineedit;
pub mod thesnapperbar;

pub mod prelude {
    pub use crate::theui::thewidget::thecolorbutton::TheColorButton;
    pub use crate::theui::thewidget::thedropdownmenu::TheDropdownMenu;
    pub use crate::theui::thewidget::thedropdownmenu::TheDropdownMenuTrait;
    pub use crate::theui::thewidget::themenubar::TheMenubar;
    pub use crate::theui::thewidget::thesectionbarbutton::TheSectionbarButton;
    pub use crate::theui::thewidget::thesectionbarbutton::TheSectionbarButtonTrait;
    pub use crate::theui::thewidget::thesectionbar::TheSectionbar;
    pub use crate::theui::thewidget::theswitchbar::{TheSwitchbar, TheSwitchbarTrait};
    pub use crate::theui::thewidget::thetext::{TheText, TheTextTrait};
    pub use crate::theui::thewidget::thesnapperbar::{TheSnapperbar, TheSnapperbarTrait};

    pub use crate::theui::thewidget::thetextlineedit::{TheTextLineEdit, TheTextLineEditTrait};

    pub use crate::theui::thewidget::TheWidget;
    pub use crate::theui::thewidget::TheWidgetId;
    pub use crate::theui::thewidget::TheWidgetState;
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TheWidgetState {
    None,
    Clicked,
    Selected,
}

/// TheWidget trait defines the asbtract functionality of a widget.
#[allow(unused)]
pub trait TheWidget {
    fn new(name: String) -> Self
    where
        Self: Sized;

    fn id(&self) -> &TheWidgetId;

    /// Called during layouts to give Widgets a chance to dynamically change size (for example for when a widgets text changes). The function is supposed to adjust its limiter.
    fn calculate_size(&mut self, ctx: &mut TheContext) {}

    /// Returns a reference to the dimensions of the widget.
    fn dim(&self) -> &TheDim;

    /// Returns a mutable reference to the dimensions of the widget.
    fn dim_mut(&mut self) -> &mut TheDim;

    /// Returns a reference to the size limiter of the widget.
    fn limiter(&self) -> &TheSizeLimiter;

    /// Returns a mutable reference to the limiter of the widget.
    fn limiter_mut(&mut self) -> &mut TheSizeLimiter;

    /// Set the dimensions of the widget
    fn set_dim(&mut self, dim: TheDim) {}

    /// Returns the current state of the widget.
    fn state(&self) -> TheWidgetState {
        TheWidgetState::None
    }

    /// Returns the current open state of the widget.
    fn is_open(&self) -> bool {
        false
    }

    /// Set the widget state.
    fn set_state(&mut self, state: TheWidgetState) {}

    /// Set the widget value.
    fn set_value(&mut self, value: TheValue) {}

    /// Draw the widget in the given style
    fn draw(
        &mut self,
        buffer: &mut TheRGBABuffer,
        style: &mut Box<dyn TheStyle>,
        ctx: &mut TheContext,
    ) {
    }

    /// Draw the widget in the given style
    fn draw_overlay(
        &mut self,
        style: &mut Box<dyn TheStyle>,
        ctx: &mut TheContext,
    ) -> TheRGBABuffer {
        TheRGBABuffer::empty()
    }

    fn update(&mut self, ctx: &mut TheContext) {}

    /// Widgets who supports hover return true
    fn supports_hover(&mut self) -> bool {
        false
    }

    /// If this function returns true it indicates that the widget needs a redraw.
    fn needs_redraw(&mut self) -> bool {
        false
    }

    fn set_needs_redraw(&mut self, redraw: bool) {}

    /// Process an user driven device event, returns true if we need to redraw.
    fn on_event(&mut self, event: &TheEvent, ctx: &mut TheContext) -> bool {
        false
    }
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
