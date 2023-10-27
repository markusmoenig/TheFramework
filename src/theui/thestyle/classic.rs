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
        widget: &mut dyn TheWidget,
        shrinker: &mut TheDimShrinker,
        ctx: &mut TheContext,
    ) {
        let stride = buffer.stride();

        let border_color = if widget.id().equals(&ctx.ui.focus) {
            self.theme().color(SelectedWidgetBorder)
        } else {
            self.theme().color(DefaultWidgetBorder)
        };

        ctx.draw.rect_outline(
            buffer.pixels_mut(),
            &widget.dim().to_buffer_shrunk_utuple(shrinker),
            stride,
            border_color,
        );

        shrinker.shrink(2);
    }

    fn draw_text_edit_border(
        &mut self,
        buffer: &mut TheRGBABuffer,
        widget: &mut dyn TheWidget,
        shrinker: &mut TheDimShrinker,
        ctx: &mut TheContext,
    ) {
        let stride = buffer.stride();

        if widget.id().equals(&ctx.ui.focus) {
            ctx.draw.rect_outline_border(
                buffer.pixels_mut(),
                &widget.dim().to_buffer_shrunk_utuple(shrinker),
                stride,
                self.theme().color(SelectedTextEditBorder1),
                2,
            );

            shrinker.shrink(1);

            ctx.draw.rect_outline(
                buffer.pixels_mut(),
                &widget.dim().to_buffer_shrunk_utuple(shrinker),
                stride,
                self.theme().color(SelectedTextEditBorder2),
            );
            shrinker.shrink(1);
        } else {
            shrinker.shrink(1);
            ctx.draw.rect_outline_border(
                buffer.pixels_mut(),
                &widget.dim().to_buffer_shrunk_utuple(shrinker),
                stride,
                self.theme().color(TextEditBorder),
                1,
            );
            shrinker.shrink(1);
        }
    }
}
