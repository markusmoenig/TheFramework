use crate::prelude::*;

pub struct TheClassicStyle {

}

/// Implements TheStyle trait for the default Classic look.
impl TheStyle for TheClassicStyle {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {

        }
    }

    /// Draw the widget border
    fn draw_widget_border(&mut self, buffer: &mut TheRGBABuffer, dim: &TheDim, shrinker: &mut TheDimShrinker, ctx: &mut TheContext) {
        let stride = buffer.stride();
        ctx.draw.rect_outline(
            buffer.pixels_mut(),
            &dim.to_shrunk_utuple(shrinker),
            stride,
            [128, 128, 128, 255],
        );

        shrinker.shrink(2);
    }
}