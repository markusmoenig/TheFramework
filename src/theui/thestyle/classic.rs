use crate::prelude::*;

pub struct TheClassicStyle {
    dark: Box<dyn TheTheme>,
}

/// Implements TheStyle trait for the default Classic look.
impl TheStyle for TheClassicStyle {
    fn new() -> Self
    where
        Self: Sized,
    {
        let dark = Box::new(TheDarkTheme::new());
        Self { dark }
    }

    fn theme(&self) -> &Box<dyn TheTheme> {
        &self.dark
    }

    fn draw_widget_border(
        &mut self,
        buffer: &mut TheRGBABuffer,
        dim: &TheDim,
        shrinker: &mut TheDimShrinker,
        ctx: &mut TheContext,
    ) {
        let stride = buffer.stride();
        ctx.draw.rect_outline(
            buffer.pixels_mut(),
            &dim.to_local_shrunk_utuple(shrinker),
            stride,
            self.theme().color(DefaultWidgetBorder),
        );

        shrinker.shrink(2);
    }
}
