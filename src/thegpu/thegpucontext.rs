use crate::prelude::*;

#[allow(unused)]
pub trait TheGpuContext {
    type Error: std::error::Error;
    type Layer;
    type LayerId;
    type ShaderInfo;
    type Surface;
    type TextureId;

    fn add_layer(&mut self) -> Self::LayerId {
        unimplemented!("Won't support");
    }

    fn draw(&self) -> Result<Option<Vec<u8>>, Self::Error>;

    fn layer(&self, layer_id: Self::LayerId) -> Option<&Self::Layer> {
        unimplemented!("Won't support");
    }

    fn layer_mut(&mut self, layer_id: Self::LayerId) -> Option<&mut Self::Layer>;

    fn load_texture(&mut self, width: u32, height: u32, buffer: &[u8]) -> Self::TextureId {
        unimplemented!("Won't support");
    }

    fn place_texture(
        &mut self,
        layer_id: Self::LayerId,
        texture_id: Self::TextureId,
        coord: Vec2<f32>,
    ) {
        unimplemented!("Won't support");
    }

    fn remove_layer(&mut self, layer_id: Self::LayerId) -> Option<Self::Layer> {
        unimplemented!("Won't support");
    }

    fn request_capture(&mut self, capture: bool) {
        unimplemented!("Won't support");
    }

    fn resize(&mut self, width: u32, height: u32);

    fn rotate(&mut self, theta: f32);

    fn scale(&mut self, scale: f32);

    fn set_compute_shader(&mut self, shader: Self::ShaderInfo) {
        unimplemented!("Won't support");
    }

    fn set_fragment_shader(&mut self, shader: Self::ShaderInfo) {
        unimplemented!("Won't support");
    }

    fn set_surface(&mut self, width: u32, height: u32, surface: Self::Surface);

    fn set_vertex_shader(&mut self, shader: Self::ShaderInfo) {
        unimplemented!("Won't support");
    }

    fn translate(&mut self, x: f32, y: f32);

    fn translate_coord_to_local(&self, x: u32, y: u32) -> (u32, u32);

    fn unload_texture(&mut self, texture_id: Self::TextureId) {
        unimplemented!("Won't support");
    }
}
