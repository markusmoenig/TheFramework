use crate::prelude::*;

pub mod hlayout;
pub mod vlayout;

pub mod prelude {
    pub use crate::theui::thelayout::hlayout::TheHLayout;
    pub use crate::theui::thelayout::vlayout::TheVLayout;

    pub use crate::theui::thelayout::TheLayout;
}

/// TheLayout trait defines an abstract layout interface for widgets.
#[allow(unused)]
pub trait TheLayout {
    fn new(name: String) -> Self
    where
        Self: Sized;

    /// Returns the id of the layout.
    fn id(&self) -> &TheWidgetId;

    /// Returns a reference to the dimensions of the widget.
    fn dim(&self) -> &TheDim;

    /// Returns a mutable reference to the dimensions of the widget.
    fn dim_mut(&mut self) -> &mut TheDim;

    /// Set the dimensions of the widget
    fn set_dim(&mut self, dim: TheDim) {}

    /// Returns a reference to the size limiter of the widget.
    fn limiter(&self) -> &TheSizeLimiter;

    /// Returns a mutable reference to the limiter of the widget.
    fn limiter_mut(&mut self) -> &mut TheSizeLimiter;

    /// Sets the margin for content in the layout
    fn set_margin(&mut self, margin: Vec4i) {}

    /// Set the padding for content in the layout
    fn set_padding(&mut self, padding: i32) {}

    /// Set the background color for the layout
    fn set_background_color(&mut self, color: Option<TheThemeColors>) {}

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