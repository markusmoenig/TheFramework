use crate::prelude::*;

pub trait TheComputePass {
    fn compute(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<(), TheGpuContextError>;
}
