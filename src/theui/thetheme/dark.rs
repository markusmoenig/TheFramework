use crate::prelude::*;

use super::TheThemeColors;

pub struct TheDarkTheme {
    colors: FxHashMap<TheThemeColors, RGBA>
}

/// Implements TheDarkTheme
impl TheTheme for TheDarkTheme {
    fn new() -> Self
    where
        Self: Sized,
    {
        let mut colors = FxHashMap::default();

        colors.insert(DefaultWidgetBorder, [128, 128, 128, 255]);

        Self {
            colors
        }
    }

    fn color(&self, of: TheThemeColors) -> &RGBA {
        if let Some(color) = self.colors.get(&of) {
            color
        } else {
            &[0, 0, 0, 255]
        }
    }
}
