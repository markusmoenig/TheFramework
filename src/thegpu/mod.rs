mod thecomputepass;
mod thegpucontext;
mod therenderpass;
mod thetexturerenderpass;

pub mod prelude {
    pub use crate::thegpu::thecomputepass::TheComputePass;
    pub use crate::thegpu::thegpucontext::{TheGpuContext, TheGpuContextError};
    pub use crate::thegpu::therenderpass::TheRenderPass;
    pub use crate::thegpu::thetexturerenderpass::{
        TheTextureId, TheTextureRenderLayer, TheTextureRenderPass,
    };
}
