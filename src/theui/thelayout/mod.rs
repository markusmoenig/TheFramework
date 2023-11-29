use crate::prelude::*;

#[cfg(feature = "code")]
pub mod thecodelayout;
pub mod thehlayout;
pub mod thelistlayout;
pub mod thergbalayout;
pub mod thesnapperlayout;
pub mod thestacklayout;
pub mod thetablayout;
pub mod thetextlayout;
pub mod thevlayout;
pub mod thesharedlayout;

pub mod prelude {
    #[cfg(feature = "code")]
    pub use crate::theui::thelayout::thecodelayout::{TheCodeLayout, TheCodeLayoutTrait};
    pub use crate::theui::thelayout::thehlayout::{TheHLayout, TheHLayoutMode, TheHLayoutTrait};
    pub use crate::theui::thelayout::thelistlayout::{TheListLayout, TheListLayoutTrait};
    pub use crate::theui::thelayout::thergbalayout::{TheRGBALayout, TheRGBALayoutTrait};
    pub use crate::theui::thelayout::thesnapperlayout::{TheSnapperLayout, TheSnapperLayoutTrait};
    pub use crate::theui::thelayout::thestacklayout::{TheStackLayout, TheStackLayoutTrait};
    pub use crate::theui::thelayout::thetablayout::{TheTabLayout, TheTabLayoutTrait};
    pub use crate::theui::thelayout::thetextlayout::{TheTextLayout, TheTextLayoutTrait};
    pub use crate::theui::thelayout::thevlayout::{TheVLayout, TheVLayoutTrait};
    pub use crate::theui::thelayout::thesharedlayout::{TheSharedLayout, TheSharedLayoutTrait, TheSharedLayoutMode};

    pub use crate::theui::thelayout::TheLayout;
}

/// TheLayout trait defines an abstract layout interface for widgets.
#[allow(unused)]
pub trait TheLayout {
    fn new(id: TheId) -> Self
    where
        Self: Sized;

    /// Returns the id of the layout.
    fn id(&self) -> &TheId;

    /// Returns a reference to the dimensions of the widget.
    fn dim(&self) -> &TheDim;

    /// Returns a mutable reference to the dimensions of the widget.
    fn dim_mut(&mut self) -> &mut TheDim;

    /// Set the dimensions of the widget
    fn set_dim(&mut self, dim: TheDim, ctx: &mut TheContext) {}

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

    /// If this function returns true it indicates that the layout needs a redraw.
    fn needs_redraw(&mut self) -> bool {
        for w in self.widgets() {
            if w.needs_redraw() {
                return true;
            }
        }
        false
    }

    /// Relayouts the layout.
    fn relayout(&mut self, ctx: &mut TheContext) {}

    fn get_layout(
        &mut self,
        name: Option<&String>,
        uuid: Option<&Uuid>,
    ) -> Option<&mut Box<dyn TheLayout>> {
        None
    }

    fn get_widget(
        &mut self,
        name: Option<&String>,
        uuid: Option<&Uuid>,
    ) -> Option<&mut Box<dyn TheWidget>>;

    fn get_widget_at_coord(&mut self, coord: Vec2i) -> Option<&mut Box<dyn TheWidget>>;

    fn widgets(&mut self) -> &mut Vec<Box<dyn TheWidget>>;

    /// Draw the widget in the given style
    fn draw(
        &mut self,
        buffer: &mut TheRGBABuffer,
        style: &mut Box<dyn TheStyle>,
        ctx: &mut TheContext,
    );

    /// Attempts to cast to TheStackLayoutTrait. Only valid for TheStackLayout.
    fn as_stack_layout(&mut self) -> Option<&mut dyn TheStackLayoutTrait> {
        None
    }

    /// Attempts to cast to TheListLayoutTrait. Only valid for TheListLayout.
    fn as_list_layout(&mut self) -> Option<&mut dyn TheListLayoutTrait> {
        None
    }

    /// Attempts to cast to TheRGBALayoutTrait. Only valid for TheRGBALayout.
    fn as_rgba_layout(&mut self) -> Option<&mut dyn TheRGBALayoutTrait> {
        None
    }

    /// Attempts to cast to TheTabLayoutTrait. Only valid for TheTabLayout.
    fn as_tab_layout(&mut self) -> Option<&mut dyn TheTabLayoutTrait> {
        None
    }

    /// Attempts to cast to TheSharedLayoutTrait. Only valid for TheSharedLayout.
    fn as_shared_layout(&mut self) -> Option<&mut dyn TheSharedLayoutTrait> {
        None
    }

    /// Attempts to cast to TheCodeLayoutTrait. Only valid for TheCodeLayout.
    #[cfg(feature = "code")]
    fn as_code_layout(&mut self) -> Option<&mut dyn TheCodeLayoutTrait> {
        None
    }
}
