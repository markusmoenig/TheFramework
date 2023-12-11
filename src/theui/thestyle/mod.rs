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
    fn theme(&mut self) -> &mut Box<dyn TheTheme>;

    /// Draw the widget border
    fn draw_widget_border(
        &mut self,
        buffer: &mut TheRGBABuffer,
        widget: &mut dyn TheWidget,
        shrinker: &mut TheDimShrinker,
        ctx: &mut TheContext,
    ) {
    }

    /// Draw the widget border
    fn draw_text_edit_border(
        &mut self,
        buffer: &mut TheRGBABuffer,
        widget: &mut dyn TheWidget,
        shrinker: &mut TheDimShrinker,
        ctx: &mut TheContext,
        draw_focus: bool,
        disabled: bool,
    ) {
    }

    /// Creates a preview image for the drop.
    fn create_drop_image(&self, drop: &mut TheDrop, ctx: &mut TheContext) {
        let mut buffer = TheRGBABuffer::new(TheDim::new(0, 0, 120, 20));

        let utuple = buffer.dim().to_buffer_utuple();
        let stride = buffer.stride();

        ctx.draw.rect(buffer.pixels_mut(), &utuple, stride, &WHITE);

        drop.set_image(buffer);
    }
}
