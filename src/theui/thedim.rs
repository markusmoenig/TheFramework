use crate::prelude::*;

#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub struct TheDim {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub root: bool,
}

impl TheDim {
    pub fn zero() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            root: false,
        }
    }

    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            root: false,
        }
    }

    pub fn coordinate(&self) -> Vec2i {
        Vec2i::new(self.x, self.y)
    }

    /// Check for size validity
    pub fn is_valid(&self) -> bool {
        self.width > 0 && self.height > 0
    }

    /// Checks if the given coordinate is inside the dimension.
    pub fn contains(&self, coord: Vec2i) -> bool {
        self.x <= coord.x
            && self.x + self.width > coord.x
            && self.y <= coord.y
            && self.y + self.height > coord.y
    }

    /// Returns the given screen coordinate as a local coordinate.
    pub fn to_local(&self, coord: Vec2i) -> Vec2i {
        coord - self.coordinate()
    }

    /// Returns the dimension as an usize tuple (used by the drawing routines)
    pub fn to_utuple(&self) -> (usize, usize, usize, usize) {
        if self.root {
            (
                self.x as usize,
                self.y as usize,
                self.width as usize,
                self.height as usize,
            )
        } else {
            self.to_zero_based_utuple()
        }
    }

    /// Returns the dimension as an usize tuple (used by the drawing routines)
    pub fn to_shrunk_utuple(&self, shrinker: &TheDimShrinker) -> (usize, usize, usize, usize) {
        if self.root {
            (
                (self.x + shrinker.left) as usize,
                (self.y + shrinker.top) as usize,
                (self.width - shrinker.right) as usize,
                (self.height - shrinker.bottom) as usize,
            )
        } else {
            (
                shrinker.left as usize,
                shrinker.top as usize,
                (self.width - shrinker.right) as usize,
                (self.height - shrinker.bottom) as usize,
            )
        }
    }

    /// Returns the zero based dimensions as an usize tuple (used by the drawing routines)
    pub fn to_zero_based_utuple(&self) -> (usize, usize, usize, usize) {
        (0, 0, self.width as usize, self.height as usize)
    }
}

/// Shrink content of TheDim, used in styles to provide a way to implement custom sized borders for widgets.
#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub struct TheDimShrinker {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl TheDimShrinker {
    pub fn zero() -> Self {
        Self {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        }
    }

    /// Shrink by the given value
    pub fn shrink(&mut self, value: i32) {
        self.left += value;
        self.top += value;
        self.right += value * 2;
        self.bottom += value * 2;
    }

    /// Shrink by the given amounts.
    pub fn shrink_by(&mut self, left: i32, top: i32, right: i32, bottom: i32) {
        self.left += left;
        self.top += top;
        self.right += right * 2;
        self.bottom += bottom * 2;
    }
}
