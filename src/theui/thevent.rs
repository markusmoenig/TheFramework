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
    ModifierChanged(bool, bool, bool, bool),

    DropPreview(Vec2i, TheDrop),
    Drop(Vec2i, TheDrop),

    // These events define widget states.
    StateChanged(TheId, TheWidgetState),
    SetState(String, TheWidgetState),

    DragStarted(TheId, String, Vec2i),
    DragStartedWithNoImage(TheDrop),

    ValueChanged(TheId, TheValue),
    SetValue(Uuid, TheValue),
    ScrollBy(TheId, Vec2i),

    GainedFocus(TheId),
    LostFocus(TheId),
    GainedHover(TheId),
    LostHover(TheId),
    SizeChanged(TheId),

    RedirectWidgetValueToLayout(TheId, TheId, TheValue),

    SetStatusText(TheId, String),

    // Tabbar, Groupbutton
    IndexChanged(TheId, usize),

    // The index of the palette has changed.
    PaletteIndexChanged(TheId, u16),
    ColorButtonClicked(TheId),

    // Tile / Code Editor
    TileSelectionChanged(TheId),
    TilePicked(TheId, Vec2i),
    TileEditorClicked(TheId, Vec2i),
    TileEditorDragged(TheId, Vec2i),
    TileEditorHoverChanged(TheId, Vec2i),
    TileEditorDrop(TheId, Vec2i, TheDrop),
    TileEditorDelete(TheId, FxHashSet<(i32, i32)>),
    TileEditorUp(TheId),

    RenderViewClicked(TheId, Vec2i),
    RenderViewDragged(TheId, Vec2i),
    RenderViewHoverChanged(TheId, Vec2i),
    RenderViewLostHover(TheId),
    RenderViewScrollBy(TheId, Vec2i),

    // CodeEditor
    #[cfg(feature = "code")]
    CodeEditorSelectionChanged(TheId, Option<(u16, u16)>),
    #[cfg(feature = "code")]
    CodeEditorChanged(TheId, TheCodeGrid),
    #[cfg(feature = "code")]
    CodeBundleChanged(TheCodeBundle, bool),

    // Timeline
    TimelineMarkerSelected(TheId, TheTime),

    // SDF
    SDFIndexChanged(TheId, u32),

    // Show the given context menu at the given (global) coordinates.
    ShowContextMenu(TheId, Vec2i, TheContextMenu),
    ShowMenu(TheId, Vec2i, TheContextMenu),
    ContextMenuSelected(TheId, TheId),
    ContextMenuClosed(TheId),

    // Nodes
    NodeSelectedIndexChanged(TheId, Option<usize>),
    NodeDragged(TheId, usize, Vec2i),
    NodeConnectionAdded(TheId, Vec<(u16, u8, u16, u8)>),
    NodeConnectionRemoved(TheId, Vec<(u16, u8, u16, u8)>),
    NodeDeleted(TheId, usize, Vec<(u16, u8, u16, u8)>),

    //
    DialogValueOnClose(TheDialogButtonRole, String, Uuid, TheValue),

    // These events define layout states.
    SetStackIndex(TheId, usize),
    NewListItemSelected(TheId, TheId),
    ScrollLayout(TheId, Vec2i),

    // Utility
    FileRequesterResult(TheId, Vec<std::path::PathBuf>),
    ImageDecodeResult(TheId, String, TheRGBABuffer),

    // The top canvas has been resized.
    Resize,

    // Custom event for applications.
    Custom(TheId, TheValue),
}
