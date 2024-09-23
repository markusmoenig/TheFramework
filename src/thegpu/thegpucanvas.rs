use crate::prelude::*;

pub struct TheGpuCanvas<C: TheGpuContext> {
    context: C,

    width: u32,
    height: u32,
}

impl<C: TheGpuContext> TheGpuCanvas<C> {
    pub fn new(context: C) -> Self {
        Self {
            context,
            width: 0,
            height: 0,
        }
    }

    pub fn add_layer(&mut self, label: Option<&str>) -> C::LayerId {
        self.context.add_layer(label)
    }

    pub fn clear(&mut self) {
        self.context.clear();
    }

    pub fn context(&self) -> &C {
        &self.context
    }

    // Draw into surface directly
    pub fn draw(&self) -> Result<(), C::Error> {
        self.context.draw()
    }

    pub fn layer(&self, layer_id: C::LayerId) -> Option<&C::Layer> {
        self.context.layer(layer_id)
    }

    pub fn layer_mut(&mut self, layer_id: C::LayerId) -> Option<&mut C::Layer> {
        self.context.layer_mut(layer_id)
    }

    pub fn load_texture(&mut self, width: u32, height: u32, buffer: &[u8]) -> C::TextureId {
        self.context.load_texture(width, height, buffer)
    }

    pub fn place_texture(
        &mut self,
        layer_id: C::LayerId,
        texture_id: C::TextureId,
        coord: Vec2<f32>,
    ) {
        self.context.place_texture(layer_id, texture_id, coord);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.context.resize(width, height);
    }

    pub fn scale(&mut self, scale: f64) {
        self.context.scale(scale);
    }

    pub fn scale_layer(&mut self, layer_id: C::LayerId, scale: f64) {
        self.context.scale_layer(layer_id, scale);
    }

    pub fn set_surface(&mut self, width: u32, height: u32, target: impl Into<C::Surface>) {
        self.context.set_surface(width, height, target.into());
    }

    pub fn set_surface_with<F>(&mut self, width: u32, height: u32, f: F)
    where
        F: FnOnce(&C) -> C::Surface,
    {
        self.context.set_surface(width, height, f(&self.context));
    }

    // Copy data into an existing buffer
    pub fn to_buffer(&self, dst_buffer: &mut TheRGBABuffer) {}

    // Copy data into a new buffer
    pub fn to_new_buffer(&self) -> TheRGBABuffer {
        TheRGBABuffer::from(self.context.buffer().to_owned(), self.width, self.height)
    }

    pub fn translate_coord_to_local(&self, x: u32, y: u32) -> (u32, u32) {
        self.context.translate_coord_to_local(x, y)
    }

    pub fn unload_texture(&mut self, texture_id: C::TextureId) {
        self.context.unload_texture(texture_id)
    }
}
