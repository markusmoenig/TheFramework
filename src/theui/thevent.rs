use crate::prelude::*;

/// All events which are handled by the framework
#[derive(Clone, Debug)]
pub enum TheEvent {
    // These events are passed to the on_event function of the widgets and cover user interaction.
    MouseDown(TheValue),

    // These events define widget states.
    Focus(TheWidgetId),
    LostFocus(TheWidgetId),
}
