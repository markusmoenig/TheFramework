use crate::prelude::*;
use rayon::prelude::*;

pub struct TheSDFCanvas {
    pub background: crate::thecolor::TheColor,
    pub highlight: crate::thecolor::TheColor,

    pub selected: Option<usize>,
    pub hover: Option<usize>,

    pub sdfs: Vec<TheSDF>,
    pub patterns: Vec<ThePattern>,
}

impl Default for TheSDFCanvas {
    fn default() -> Self {
        Self::new()
    }
}

impl TheSDFCanvas {
    pub fn new() -> Self {
        Self {
            sdfs: vec![],
            patterns: vec![],

            selected: None,
            hover: None,

            background: TheColor::black(),
            highlight: TheColor::white(),
        }
    }

    /// Adds an SDF to the canvas.
    pub fn add(&mut self, sdf: TheSDF, pattern: ThePattern) {
        self.sdfs.push(sdf);
        self.patterns.push(pattern);
    }

    /// Renders the sdfs into the given buffer.
    pub fn render(&self, buffer: &mut TheRGBABuffer) {
        let width = buffer.dim().width as usize;
        let height = buffer.dim().height;

        let pixels = buffer.pixels_mut();

        pixels
            .par_rchunks_exact_mut(width * 4)
            .enumerate()
            .for_each(|(j, line)| {
                for (i, pixel) in line.chunks_exact_mut(4).enumerate() {
                    let i = j * width + i;

                    let x = (i % width) as i32;
                    let y = height - (i / width) as i32 - 1;

                    let mut color = self.background.clone();

                    for (index, sdf) in self.sdfs.iter().enumerate() {
                        let p = vec2f(x as f32, y as f32);
                        let d = sdf.distance(p);

                        let c = self.patterns[index].get_color(p, &d, self.highlight(index));
                        color = color.mix(&c, self.fill_mask(d));
                    }

                    pixel.copy_from_slice(&color.to_u8_array());
                }
            });
    }

    /// Returns the index of the sdf at the given position.
    pub fn index_at(&self, p: Vec2f) -> Option<usize> {
        for (index, sdf) in self.sdfs.iter().enumerate() {
            let d = sdf.distance(p);
            if d < 0.0 {
                return Some(index);
            }
        }
        None
    }

    /// Returns the fill mask for the given distance.
    #[inline(always)]
    fn fill_mask(&self, dist: f32) -> f32 {
        (-dist).clamp(0.0, 1.0)
    }

    /// Returns the selected color if the given sdf index is highlighted.
    #[inline(always)]
    fn highlight(&self, index: usize) -> Option<&TheColor> {
        if self.selected == Some(index) || self.hover == Some(index) {
            Some(&self.highlight)
        } else {
            None
        }
    }

    /// Clear the canvas.
    pub fn clear(&mut self) {
        self.sdfs.clear();
        self.patterns.clear();
        self.selected = None;
        self.hover = None;
    }

    /// Returns true if the canvas is empty.
    pub fn is_empty(&self) -> bool {
        self.sdfs.is_empty()
    }
}
