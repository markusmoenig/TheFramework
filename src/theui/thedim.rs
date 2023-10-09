#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub struct TheDim {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub root:  bool,
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

    /// Check for size validity
    pub fn is_valid(&self) -> bool {
        if self.height > 0 && self.height > 0 {
            true
        } else {
            false
        }
    }

    /// Returns the dimension as an usize tuple (used by the drawing routines)
    pub fn to_utuple(&self) -> (usize, usize, usize, usize) {
        if self.root {
            (self.x as usize, self.y as usize, self.width as usize, self.height as usize)
        } else {
            self.to_zero_based_utuple()
        }
    }

    /// Returns the zero based dimensions as an usize tuple (used by the drawing routines)
    pub fn to_zero_based_utuple(&self) -> (usize, usize, usize, usize) {
        (0, 0, self.width as usize, self.height as usize)
    }
}
