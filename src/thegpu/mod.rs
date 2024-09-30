mod thegpucontext;

#[cfg(feature = "pixels_gpu")]
mod thepixelscontext;

pub mod prelude {
    pub use crate::thegpu::thegpucontext::TheGpuContext;

    #[cfg(feature = "pixels_gpu")]
    pub use crate::thegpu::thepixelscontext::ThePixelsContext;
}
