use crate::prelude::*;

pub mod dark;

pub mod prelude {
    pub use crate::theui::thetheme::dark::TheDarkTheme;
}

/// TheTheme defines all colors and other attributes of a theme.
#[allow(unused)]
pub trait TheTheme {
    fn new() -> Self
    where
        Self: Sized;

    /// Returns the color of the given theme color.
    fn color(&self, of: TheThemeColors) -> &RGBA;
}

/// The
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum TheThemeColors {
    DefaultWidgetBorder,
    SelectedWidgetBorder,
    DefaultWidgetBackground,

    SwitchbarBorder,

    SectionbarHeaderBorder,
    SectionbarBackground,
    SectionbarNormalTextColor,
    SectionbarSelectedTextColor,

    TextLayoutBackground,
    TextLayoutBorder,

    TextEditBackground,
    TextEditBorder,
    SelectedTextEditBorder1,
    SelectedTextEditBorder2,
    TextEditTextColor,
    TextEditCursorColor,

    MenubarPopupBackground,
    MenubarPopupBorder,

    SliderSmallColor1,
    SliderSmallColor2,
    SliderSmallColor3,
    SliderSmallColor4,

    MenubarButtonHover,
    MenubarButtonHoverBorder,
    MenubarButtonClicked,
    MenubarButtonClickedBorder,

    MenubarButtonSeparator1,
    MenubarButtonSeparator2,

    ToolbarButtonNormal,
    ToolbarButtonNormalBorder,
    ToolbarButtonHover,
    ToolbarButtonHoverBorder,
    ToolbarButtonClicked,
    ToolbarButtonClickedBorder,

    TraybarButtonNormal,
    TraybarButtonNormalBorder,
    TraybarButtonHover,
    TraybarButtonHoverBorder,
    TraybarButtonClicked,
    TraybarButtonClickedBorder,

    ListLayoutBackground,
    ListItemNormal,
    ListItemSelected,
    ListItemHover,
    ListItemText,
    ListItemIconBorder,

    ScrollbarBackground,
    ScrollbarSeparator,

    TabbarBackground,
    TabbarConnector,
    TabbarText,

    TraybarBorder,
    TraybarBackground,
    TraybarBottomBorder,

    StatusbarStart,
    StatusbarEnd,
}
