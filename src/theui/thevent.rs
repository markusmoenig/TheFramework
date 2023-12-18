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
    MouseWheel(Vec2i),

    KeyDown(TheValue),
    KeyCodeDown(TheValue),

    DropPreview(Vec2i, TheDrop),
    Drop(Vec2i, TheDrop),

    // These events define widget states.
    StateChanged(TheId, TheWidgetState),
    SetState(String, TheWidgetState),

    DragStarted(TheId),

    ValueChanged(TheId, TheValue),
    SetValue(Uuid, TheValue),
    ScrollBy(TheId, Vec2i),

    GainedFocus(TheId),
    LostFocus(TheId),
    GainedHover(TheId),
    LostHover(TheId),

    SetStatusText(TheId, String),

    // Tabbar, Groupbutton
    IndexChanged(TheId, usize),

    // Tile / Code Editor
    TileSelectionChanged(TheId),
    TileEditorClicked(TheId, Vec2i),
    TileEditorHoverChanged(TheId, Vec2i),

    CodeEditorSelectionChanged(TheId, Option<(u16, u16)>),
    CodeEditorChanged(TheId, TheCodeGrid),
    CodeBundleChanged(TheCodeBundle),

    // Show the given context menu at the given coordinates
    ShowContextMenu(TheId, Vec2i),

    // These events define layout states.
    SetStackIndex(TheId, usize),
    NewListItemSelected(TheId, TheId),

    // Utility
    FileRequesterResult(TheId, Vec<std::path::PathBuf>),
    ImageDecodeResult(TheId, String, TheRGBABuffer),
}
