use crate::prelude::*;

pub trait TheRenderPass {
    fn draw(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        scale_factor: f32,
        surface_texture: &wgpu::SurfaceTexture,
        surface_config: &wgpu::SurfaceConfiguration,
    ) -> Result<(), TheGpuContextError>;
}
