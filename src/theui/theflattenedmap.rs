#[derive(Clone, Debug)]
pub struct TheFlattenedMap<T>
where
    T: Clone,
{
    data: Vec<Option<T>>,
    width: i32,
    height: i32,
}

impl<T> TheFlattenedMap<T>
where
    T: Clone,
{
    /// Creates a new `TheFlattenedMap` with specified width and height.
    pub fn new(width: i32, height: i32) -> Self {
        TheFlattenedMap {
            data: vec![None; (width * height) as usize],
            width,
            height,
        }
    }

    /// Converts a 2D key into a 1D index.
    fn key_to_index(&self, key: (i32, i32)) -> Option<usize> {
        let (x, y) = key;
        if x >= 0 && x < self.width && y >= 0 && y < self.height {
            Some((y * self.width + x) as usize)
        } else {
            None
        }
    }

    /// Sets the value for a key.
    pub fn set(&mut self, key: (i32, i32), value: T) {
        if let Some(index) = self.key_to_index(key) {
            self.data[index] = Some(value);
        }
    }

    /// Gets a reference to the value for a key, if it exists.
    pub fn get(&self, key: (i32, i32)) -> Option<&T> {
        self.key_to_index(key)
            .and_then(|index| self.data[index].as_ref())
    }

    /// Gets a mutable reference to the value for a key, if it exists.
    pub fn get_mut(&mut self, key: (i32, i32)) -> Option<&mut T> {
        if let Some(index) = self.key_to_index(key) {
            self.data[index].as_mut()
        } else {
            None
        }
    }

    /// Returns the underlying data for direct indexing.
    /// Use with caution, primarily for internal operations.
    pub fn data(&self) -> &Vec<Option<T>> {
        &self.data
    }

    /// Clears the map, setting all values to None.
    pub fn clear(&mut self) {
        for element in &mut self.data {
            *element = None;
        }
    }
}
