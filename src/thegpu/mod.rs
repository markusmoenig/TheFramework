mod thegpucontext;

#[cfg(feature = "pixels_gpu")]
mod thepixelscontext;
#[cfg(feature = "wgpu_gpu")]
mod thewgpucontext;

pub mod prelude {
    pub use crate::thegpu::thegpucontext::TheGpuContext;

    #[cfg(feature = "pixels_gpu")]
    pub use crate::thegpu::thepixelscontext::ThePixelsContext;
    #[cfg(feature = "wgpu_gpu")]
    pub use crate::thegpu::thewgpucontext::TheWgpuContext;
}
