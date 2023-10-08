#[derive(PartialEq, PartialOrd)]
pub struct TheDim<T> {
    pub x: T,
    pub y: T,
    pub width: T,
    pub height: T,
}

impl<T: Copy> TheDim<T> {
    pub fn fill(value: T) -> Self {
        Self {
            x: value,
            y: value,
            width: value,
            height: value,
        }
    }

    pub fn new(x: T, y: T, width: T, height: T) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}
