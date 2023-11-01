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

        colors.insert(SectionbarHeaderBorder, [86, 86, 86, 255]);
        colors.insert(SectionbarBackground, [130, 130, 130, 255]);
        colors.insert(SectionbarNormalTextColor, [255, 255, 255, 255]);
        colors.insert(SectionbarSelectedTextColor, [96, 96, 96, 255]);

        colors.insert(TextLayoutBackground, [82, 82, 82, 255]);
        colors.insert(TextLayoutBorder, [95, 95, 95, 255]);

        colors.insert(TextEditBackground, [148, 148, 148, 255]);
        colors.insert(SelectedTextEditBorder1, [202, 113, 230, 255]);
        colors.insert(SelectedTextEditBorder2, [187, 122, 208, 255]);
        colors.insert(TextEditBorder, [209, 209, 209, 255]);
        colors.insert(TextEditTextColor, [242, 242, 242, 255]);
        colors.insert(TextEditCursorColor, [119, 119, 119, 255]);

        colors.insert(MenubarPopupBackground, [124, 124, 124, 255]);
        colors.insert(MenubarPopupBorder, [153, 153, 153, 255]);

        colors.insert(SliderSmallColor1, [158, 158, 158, 255]);
        colors.insert(SliderSmallColor2, [174, 174, 174, 255]);
        colors.insert(SliderSmallColor3, [187, 187, 187, 255]);
        colors.insert(SliderSmallColor4, [122, 122, 122, 255]);

        colors.insert(MenubarButtonHover, [157, 157, 157, 255]);
        colors.insert(MenubarButtonHoverBorder, [179, 179, 179, 255]);
        colors.insert(MenubarButtonClicked, [149, 149, 149, 255]);
        colors.insert(MenubarButtonClickedBorder, [204, 204, 204, 255]);

        colors.insert(MenubarButtonSeparator1, [102, 102, 102, 255]);
        colors.insert(MenubarButtonSeparator2, [148, 148, 148, 255]);

        colors.insert(ListLayoutBackground, [82, 82, 82, 255]);
        colors.insert(ListItemNormal, [174, 174, 174, 255]);
        colors.insert(ListItemSelected, [187, 122, 208, 255]);
        colors.insert(ListItemHover, [237, 237, 237, 255]);
        colors.insert(ListItemText, [85, 81, 85, 255]);

        colors.insert(ScrollbarBackground, [139, 139, 139, 255]);

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
