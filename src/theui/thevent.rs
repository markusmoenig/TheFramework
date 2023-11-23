use crate::prelude::*;

/// All events which are handled by the framework
#[derive(Clone, Debug)]
pub enum TheEvent {
    // These events are passed to the on_event function of the widgets and cover user interaction.
    MouseDown(TheValue),
    Hover(TheValue),
    MouseDragged(TheValue),
    MouseUp(TheValue),

    KeyDown(TheValue),
    KeyCodeDown(TheValue),

    // These events define widget states.
    StateChanged(TheId, TheWidgetState),
    SetState(String, TheWidgetState),

    ValueChanged(TheId, TheValue),
    SetValue(Uuid, TheValue),

    GainedFocus(TheId),
    LostFocus(TheId),
    GainedHover(TheId),
    LostHover(TheId),

    TileSelectionChanged(TheId),
    TileEditorClicked(TheId, TheValue),

    // These events define layout states.
    SetStackIndex(TheId, usize),
    NewListItemSelected(TheId, TheId),

    // Utility
    FileRequesterResult(TheId, Vec<std::path::PathBuf>),
    ImageDecodeResult(TheId, String, TheRGBABuffer),
}
