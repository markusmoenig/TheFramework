use crate::prelude::*;

pub mod thecheckbutton;
#[cfg(feature = "code")]
pub mod thecodeview;
pub mod thecolorbutton;
pub mod thedropdownmenu;
pub mod thegroupbutton;
pub mod thehdivider;
pub mod thehorizontalscrollbar;
pub mod theiconview;
pub mod thelistitem;
pub mod themenubar;
pub mod themenubarbutton;
pub mod themenubarseparator;
pub mod thergbaview;
pub mod thesectionbar;
pub mod thesectionbarbutton;
pub mod theslider;
pub mod thesnapperbar;
pub mod thespacer;
pub mod thestatusbar;
pub mod theswitchbar;
pub mod thetabbar;
pub mod thetext;
// pub mod thetextedit;
pub mod thecolorpicker;
pub mod thedirectionpicker;
pub mod thesdfview;
pub mod thetextlineedit;
pub mod thetoolbar;
pub mod thetoolbarbutton;
pub mod thetraybar;
pub mod thetraybarbutton;
pub mod theverticalscrollbar;

use std::any::Any;

pub mod prelude {
    pub use crate::theui::thewidget::thecolorbutton::TheColorButton;

    pub use crate::theui::thewidget::thedropdownmenu::TheDropdownMenu;
    pub use crate::theui::thewidget::thedropdownmenu::TheDropdownMenuTrait;

    pub use crate::theui::thewidget::themenubar::TheMenubar;
    pub use crate::theui::thewidget::themenubarbutton::{TheMenubarButton, TheMenubarButtonTrait};
    pub use crate::theui::thewidget::themenubarseparator::TheMenubarSeparator;

    pub use crate::theui::thewidget::thetoolbar::TheToolbar;
    pub use crate::theui::thewidget::thetoolbarbutton::{TheToolbarButton, TheToolbarButtonTrait};

    pub use crate::theui::thewidget::thehorizontalscrollbar::{
        TheHorizontalScrollbar, TheHorizontalScrollbarTrait,
    };
    pub use crate::theui::thewidget::thelistitem::{TheListItem, TheListItemTrait};
    pub use crate::theui::thewidget::thesectionbar::TheSectionbar;
    pub use crate::theui::thewidget::thesectionbarbutton::TheSectionbarButton;
    pub use crate::theui::thewidget::thesectionbarbutton::TheSectionbarButtonTrait;
    pub use crate::theui::thewidget::theslider::{TheSlider, TheSliderTrait};
    pub use crate::theui::thewidget::thesnapperbar::{TheSnapperbar, TheSnapperbarTrait};
    pub use crate::theui::thewidget::theswitchbar::{TheSwitchbar, TheSwitchbarTrait};
    pub use crate::theui::thewidget::thetext::{TheText, TheTextTrait};
    pub use crate::theui::thewidget::theverticalscrollbar::{
        TheVerticalScrollbar, TheVerticalScrollbarTrait,
    };

    pub use crate::theui::thewidget::thegroupbutton::{TheGroupButton, TheGroupButtonTrait};

    pub use crate::theui::thewidget::thecheckbutton::TheCheckButton;
    pub use crate::theui::thewidget::thehdivider::TheHDivider;
    pub use crate::theui::thewidget::theiconview::{TheIconView, TheIconViewTrait};
    pub use crate::theui::thewidget::thergbaview::{
        TheRGBAView, TheRGBAViewMode, TheRGBAViewTrait,
    };
    pub use crate::theui::thewidget::thespacer::TheSpacer;
    pub use crate::theui::thewidget::thestatusbar::{TheStatusbar, TheStatusbarTrait};
    pub use crate::theui::thewidget::thetabbar::{TheTabbar, TheTabbarTrait};
    // pub use crate::theui::thewidget::thetextedit::{TheTextEdit, TheTextEditTrait};
    pub use crate::theui::thewidget::thesdfview::{TheSDFView, TheSDFViewTrait};
    pub use crate::theui::thewidget::thetextlineedit::{TheTextLineEdit, TheTextLineEditTrait};
    pub use crate::theui::thewidget::thetraybar::TheTraybar;
    pub use crate::theui::thewidget::thetraybarbutton::{TheTraybarButton, TheTraybarButtonTrait};

    #[cfg(feature = "code")]
    pub use crate::theui::thewidget::thecodeview::{TheCodeView, TheCodeViewTrait};

    pub use crate::theui::thewidget::thecolorpicker::{TheColorPicker, TheColorPickerTrait};
    pub use crate::theui::thewidget::thedirectionpicker::TheDirectionPicker;
    pub use crate::theui::thewidget::TheWidget;
    pub use crate::theui::thewidget::TheWidgetState;
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TheWidgetState {
    None,
    Clicked,
    Selected,
}

/// TheWidget trait defines the asbtract functionality of a widget.
#[allow(unused)]
pub trait TheWidget: Send {
    fn new(id: TheId) -> Self
    where
        Self: Sized;

    fn id(&self) -> &TheId;

    /// Called during layouts to give Widgets a chance to dynamically change size (for example for when a widgets text changes). The function is supposed to adjust its limiter.
    fn calculate_size(&mut self, ctx: &mut TheContext) {}

    /// Returns a reference to the dimensions of the widget.
    fn dim(&self) -> &TheDim;

    /// Returns a mutable reference to the dimensions of the widget.
    fn dim_mut(&mut self) -> &mut TheDim;

    /// Returns a reference to the size limiter of the widget.
    fn limiter(&self) -> &TheSizeLimiter;

    /// Returns a mutable reference to the limiter of the widget.
    fn limiter_mut(&mut self) -> &mut TheSizeLimiter;

    /// Set the dimensions of the widget
    fn set_dim(&mut self, dim: TheDim) {}

    /// Returns the current state of the widget.
    fn state(&self) -> TheWidgetState {
        TheWidgetState::None
    }

    /// Returns the current open state of the widget.
    fn is_open(&self) -> bool {
        false
    }

    /// Set the widget state.
    fn set_state(&mut self, state: TheWidgetState) {}

    /// Get the widget value.
    fn value(&self) -> TheValue {
        TheValue::Empty
    }

    /// Set the widget value.
    fn set_value(&mut self, value: TheValue) {}

    /// Retrieves the status text for the widget.
    fn status_text(&self) -> Option<String> {
        None
    }

    /// Sets the status text for the widget.
    fn set_status_text(&mut self, text: &str) {}

    /// Draw the widget in the given style
    fn draw(
        &mut self,
        buffer: &mut TheRGBABuffer,
        style: &mut Box<dyn TheStyle>,
        ctx: &mut TheContext,
    ) {
    }

    /// Draw the widget in the given style
    fn draw_overlay(
        &mut self,
        style: &mut Box<dyn TheStyle>,
        ctx: &mut TheContext,
    ) -> TheRGBABuffer {
        TheRGBABuffer::empty()
    }

    fn update(&mut self, ctx: &mut TheContext) {}

    /// Widgets who supports hover return true
    fn supports_hover(&mut self) -> bool {
        false
    }

    /// If this function returns true it indicates that the widget needs a redraw.
    fn needs_redraw(&mut self) -> bool {
        false
    }

    /// Sets the internal redraw flag of the widget to the given value.
    fn set_needs_redraw(&mut self, redraw: bool) {}

    /// Returns true if the widget is disabled
    fn disabled(&self) -> bool {
        false
    }

    /// Set the disabled state of the widget
    fn set_disabled(&mut self, disabled: bool) {}

    /// Process an user driven device event, returns true if we need to redraw.
    fn on_event(&mut self, event: &TheEvent, ctx: &mut TheContext) -> bool {
        false
    }

    /// Sets the context menu for the widget.
    fn set_context_menu(&mut self, menu: Option<TheContextMenu>) {}

    // Casting

    /// Attempts to cast to TheListItemTrait. Only valid for TheListItem.
    fn as_list_item(&mut self) -> Option<&mut dyn TheListItemTrait> {
        None
    }

    /// Attempts to cast to TheDropdownMenuTrait. Only valid for TheDropdownMenu.
    fn as_drop_down_menu(&mut self) -> Option<&mut dyn TheDropdownMenuTrait> {
        None
    }

    /// Attempts to cast to TheHorizontalScrollbarTrait. Only valid for TheHorizontalScrollbar.
    fn as_horizontal_scrollbar(&mut self) -> Option<&mut dyn TheHorizontalScrollbarTrait> {
        None
    }

    /// Attempts to cast to TheVerticalScrollbarTrait. Only valid for TheVerticalScrollbar.
    fn as_vertical_scrollbar(&mut self) -> Option<&mut dyn TheVerticalScrollbarTrait> {
        None
    }

    /// Attempts to cast to TheRGBAView. Only valid for TheVerticalScrollbar.
    fn as_rgba_view(&mut self) -> Option<&mut dyn TheRGBAViewTrait> {
        None
    }

    /// Attempts to cast to TheTextTrait. Only valid for TheText.
    fn as_text(&mut self) -> Option<&mut dyn TheTextTrait> {
        None
    }

    /// Attempts to cast to TheTabbarTrait. Only valid for TheTabbar.
    fn as_tabbar(&mut self) -> Option<&mut dyn TheTabbarTrait> {
        None
    }

    /// Attempts to cast to TheTextLineEditTrait. Only valid for TheTextLineEdit.
    fn as_text_line_edit(&mut self) -> Option<&mut dyn TheTextLineEditTrait> {
        None
    }

    /// Attempts to cast to TheIconViewTrait. Only valid for TheIconView.
    fn as_icon_view(&mut self) -> Option<&mut dyn TheIconViewTrait> {
        None
    }

    /// Attempts to cast to TheGroupButtonTrait. Only valid for TheGroupButton.
    fn as_group_button(&mut self) -> Option<&mut dyn TheGroupButtonTrait> {
        None
    }

    /// Attempts to cast to TheStatusbarTrait. Only valid for TheStatusbar.
    fn as_statusbar(&mut self) -> Option<&mut dyn TheStatusbarTrait> {
        None
    }

    /// Attempts to cast to TheMenubarButtonTrait. Only valid for TheMenubarButton.
    fn as_menubar_button(&mut self) -> Option<&mut dyn TheMenubarButtonTrait> {
        None
    }

    /// Attempts to cast to TheCodeViewTrait. Only valid for TheCodeView.
    #[cfg(feature = "code")]
    fn as_code_view(&mut self) -> Option<&mut dyn TheCodeViewTrait> {
        None
    }

    /// Cast to any
    fn as_any(&mut self) -> &mut dyn Any;
}
