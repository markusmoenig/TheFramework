use crate::prelude::*;

pub mod colorbutton;
pub mod sectionheader;
pub mod vlayout;

pub mod prelude {
    pub use crate::theui::thewidget::colorbutton::TheColorButton;
    pub use crate::theui::thewidget::sectionheader::TheSectionHeader;
    pub use crate::theui::thewidget::vlayout::TheVLayout;

    pub use crate::theui::thewidget::TheLayout;
    pub use crate::theui::thewidget::TheWidget;
    pub use crate::theui::thewidget::TheWidgetId;
}

/// TheWidget trait defines the asbtract functionality of a widget.
#[allow(unused)]
pub trait TheWidget {
    fn new(name: String) -> Self
    where
        Self: Sized;

    fn id(&self) -> &TheWidgetId;

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

    fn needs_redraw(&mut self) -> bool {
        false
    }

    fn set_needs_redraw(&mut self, redraw: bool) {}

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

/// TheLayout trait defines an abstract layout interface for widgets.
#[allow(unused)]
pub trait TheLayout {
    fn new(name: String) -> Self
    where
        Self: Sized;

    /// Returns a reference to the dimensions of the widget.
    fn dim(&self) -> &TheDim;

    /// Returns a mutable reference to the dimensions of the widget.
    fn dim_mut(&mut self) -> &mut TheDim;

    /// Set the dimensions of the widget
    fn set_dim(&mut self, dim: TheDim) {}

    fn get_widget(
        &mut self,
        name: Option<&String>,
        uuid: Option<&Uuid>,
    ) -> Option<&mut Box<dyn TheWidget>>;

    fn get_widget_at_coord(&mut self, coord: Vec2i) -> Option<&mut Box<dyn TheWidget>>;

    //fn add_widget<T: TheWidget + 'static>(&mut self, widget: T);
    fn add_widget(&mut self, widget: Box<dyn TheWidget>);
    fn widgets(&mut self) -> &mut Vec<Box<dyn TheWidget>>;

    /// Draw the widget in the given style
    fn draw(
        &mut self,
        buffer: &mut TheRGBABuffer,
        style: &mut Box<dyn TheStyle>,
        ctx: &mut TheContext,
    );
}
