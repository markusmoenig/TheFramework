use crate::prelude::*;

pub struct TheIconView {
    id: TheId,
    limiter: TheSizeLimiter,

    is_dirty: bool,
    tile: TheRGBATile,
    index: usize,

    dim: TheDim,
}

impl TheWidget for TheIconView {
    fn new(id: TheId) -> Self
    where
        Self: Sized,
    {
        let mut limiter = TheSizeLimiter::new();
        limiter.set_max_size(vec2i(24, 24));

        Self {
            id,
            limiter,

            is_dirty: true,
            tile: TheRGBATile::default(),
            index: 0,

            dim: TheDim::zero(),
        }
    }

    fn id(&self) -> &TheId {
        &self.id
    }

    fn dim(&self) -> &TheDim {
        &self.dim
    }

    fn dim_mut(&mut self) -> &mut TheDim {
        &mut self.dim
    }

    fn set_dim(&mut self, dim: TheDim) {
        if self.dim != dim {
            self.dim = dim;
        }
    }

    fn limiter(&self) -> &TheSizeLimiter {
        &self.limiter
    }

    fn limiter_mut(&mut self) -> &mut TheSizeLimiter {
        &mut self.limiter
    }

    fn needs_redraw(&mut self) -> bool {
        self.is_dirty
    }

    fn draw(
        &mut self,
        buffer: &mut TheRGBABuffer,
        _style: &mut Box<dyn TheStyle>,
        ctx: &mut TheContext,
    ) {
        let stride: usize = buffer.stride();

        if !self.dim().is_valid() {
            return;
        }

        let utuple = self.dim.to_buffer_utuple();

        if !self.tile.buffer.is_empty() {
            ctx.draw.scale_chunk(
                buffer.pixels_mut(),
                &(
                    utuple.0,
                    utuple.1,
                    self.dim.width as usize,
                    self.dim.height as usize,
                ),
                stride,
                self.tile.buffer[self.index].pixels(),
                &(self.tile.buffer[0].dim().width as usize, self.tile.buffer[0].dim().height as usize),
                1.0

            );
        }

        self.is_dirty = false;
    }

    fn as_icon_view(&mut self) -> Option<&mut dyn TheIconViewTrait> {
        Some(self)
    }
}

pub trait TheIconViewTrait {
    fn set_rgba_tile(&mut self, tile: TheRGBATile);
    fn step(&mut self);
}

impl TheIconViewTrait for TheIconView {
    fn set_rgba_tile(&mut self, tile: TheRGBATile) {
        self.tile = tile;
        self.is_dirty = true;
        self.index = 0;
    }
    fn step(&mut self) {
        if self.tile.buffer.len() >= 2 {
            self.index += 1;
            if self.index >= self.tile.buffer.len() {
                self.index = 0;
            }
            self.is_dirty = true;
        }
    }
}
