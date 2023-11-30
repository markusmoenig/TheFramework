use crate::prelude::*;

/// All events which are handled by the framework
#[derive(Clone, Debug)]
pub enum TheEvent {
    // These events are passed to the on_event function of the widgets and cover user interaction.
    Context(Vec2i),
    MouseDown(Vec2i),
    Hover(Vec2i),
    MouseDragged(Vec2i),
    MouseUp(Vec2i),

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

    // Tabbar, Groupbutton
    IndexChanged(TheId, usize),

    // Tile / Code Editor
    TileSelectionChanged(TheId),
    TileEditorClicked(TheId, TheValue),
    CodeEditorSelectionChanged(TheId, Option<(u32, u32)>),

    // Show the given context menu at the given coordinates
    ShowContextMenu(TheId, Vec2i),

    // These events define layout states.
    SetStackIndex(TheId, usize),
    NewListItemSelected(TheId, TheId),

    // Utility
    FileRequesterResult(TheId, Vec<std::path::PathBuf>),
    ImageDecodeResult(TheId, String, TheRGBABuffer),
}
