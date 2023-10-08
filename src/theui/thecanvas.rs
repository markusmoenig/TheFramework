use crate::prelude::*;

pub struct TheCanvas {
    dim: TheDim<i32>,

    min_size: Vec2<i32>,
    max_size: Vec2<i32>,

    pub left: Option<Box<TheCanvas>>,
    pub top: Option<Box<TheCanvas>>,
    pub right: Option<Box<TheCanvas>>,
    pub bottom: Option<Box<TheCanvas>>,

    pub widgets: Vec<Box<dyn TheWidget>>,
}

/// TheCanvas divides a screen dimension into 4 possible sub-spaces for its border while containing a set of widgets for its center.
impl TheCanvas {
    pub fn new() -> Self {
        Self {
            dim: TheDim::fill(0),

            min_size: Vec2::new(0, 0),
            max_size: Vec2::new(std::i32::MAX, std::i32::MAX),

            left: None,
            top: None,
            right: None,
            bottom: None,

            widgets: vec![],
        }
    }

    /// Set the dimension of the canvas
    pub fn set_dim(&mut self, dim: TheDim<i32>) {
        if dim != self.dim {
            self.dim = dim;
            self.layout();
        }
    }

    /// Sets the minimum dimensions of the canvas.
    pub fn set_min_size(&mut self, size: Vec2<i32>) {
        self.min_size = size;
    }

    /// Sets the maximum dimensions of the canvas.
    pub fn set_max_size(&mut self, size: Vec2<i32>) {
        self.max_size = size;
    }

    /// Layout the canvas according to its dimensions.
    pub fn layout(&mut self) {}
}
