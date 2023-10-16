use crate::prelude::*;

use super::TheThemeColors;

pub struct TheDarkTheme {
    colors: FxHashMap<TheThemeColors, RGBA>,
}

/// Implements TheDarkTheme
impl TheTheme for TheDarkTheme {
    fn new() -> Self
    where
        Self: Sized,
    {
        let mut colors = FxHashMap::default();

        colors.insert(DefaultWidgetBorder, [71, 71, 71, 255]);
        // colors.insert(SelectedWidgetBorder, [55, 68, 98, 255]);
        colors.insert(SelectedWidgetBorder, [249, 249, 96, 255]);
        colors.insert(DefaultWidgetBackground, [130, 130, 130, 255]);

        colors.insert(SwitchbarBorder, [81, 81, 81, 255]);

        Self { colors }
    }

    fn color(&self, of: TheThemeColors) -> &RGBA {
        if let Some(color) = self.colors.get(&of) {
            color
        } else {
            &[0, 0, 0, 255]
        }
    }
}
