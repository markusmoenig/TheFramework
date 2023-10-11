use crate::prelude::*;

pub mod classic;

pub mod prelude {
    pub use crate::theui::thestyle::classic::TheClassicStyle;
}

#[allow(unused)]
pub trait TheStyle {
    fn new() -> Self
    where
        Self: Sized;

    #[allow(clippy::borrowed_box)]
    /// Returns the current theme of the style
    fn theme(&self) -> &Box<dyn TheTheme>;

    /// Draw the widget border
    fn draw_widget_border(
        &mut self,
        buffer: &mut TheRGBABuffer,
        dim: &TheDim,
        shrinker: &mut TheDimShrinker,
        ctx: &mut TheContext,
    ) {
    }
}
