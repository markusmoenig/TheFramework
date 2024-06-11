use crate::prelude::*;

pub struct TheNodeTerminal {
    pub name: String,
    pub role: String,
    pub color: TheColor,
}

pub struct TheNode {
    pub name: String,
    pub position: Vec2i,

    pub inputs: Vec<TheNodeTerminal>,
    pub outputs: Vec<TheNodeTerminal>,

    pub preview: TheRGBABuffer,

    pub supports_preview: bool,
    pub preview_is_open: bool,
}

pub struct TheNodeCanvas {
    /// The nodes in the canvas, identified by their index.
    pub nodes: Vec<TheNode>,

    /// The width of a node.
    pub node_width: i32,

    /// The node connections: Source node index, source terminal, dest node index, dest terminal
    pub connections: Vec<(u16, u8, u16, u8)>,

    /// The scroll offset.
    pub offset: Vec2i,

    /// The currently selected node.
    pub selected_node: Option<usize>,

    /// The zoom level.
    pub zoom: f32,
}

impl Default for TheNodeCanvas {
    fn default() -> Self {
        Self::new()
    }
}

impl TheNodeCanvas {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            node_width: 128,
            connections: Vec::new(),
            offset: Vec2i::zero(),
            selected_node: None,
            zoom: 1.0,
        }
    }
}
