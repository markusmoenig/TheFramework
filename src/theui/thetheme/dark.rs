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

        colors.insert(DefaultWidgetBackground, [116, 116, 116, 255]);
        colors.insert(DefaultWidgetBorder, [146, 146, 146, 255]);
        colors.insert(SelectedWidgetBorder, [187, 122, 208, 255]);

        colors.insert(SwitchbarBorder, [86, 86, 86, 255]);

        colors.insert(SectionbarBackground, [130, 130, 130, 255]);
        colors.insert(SectionbarNormalTextColor, [255, 255, 255, 255]);
        colors.insert(SectionbarSelectedTextColor, [96, 96, 96, 255]);

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
