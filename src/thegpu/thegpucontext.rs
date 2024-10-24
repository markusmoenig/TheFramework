use std::{error::Error, fmt, iter::once, sync::mpsc::channel};

use crate::prelude::*;

const U8_SIZE: u32 = std::mem::size_of::<u8>() as u32;

#[derive(Debug)]
pub enum TheGpuContextError {
    AdapterNotFound,
    AsyncInternal { source: Box<dyn Error + 'static> },
    InvalidTextureFormat(wgpu::TextureFormat),
    RenderContextNotFound,
    WgpuInternal { source: Box<dyn Error + 'static> },
}

impl Error for TheGpuContextError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::AsyncInternal { source } => Some(source.as_ref()),
            Self::WgpuInternal { source } => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl fmt::Display for TheGpuContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AdapterNotFound => write!(
                f,
                "No adapters are found that suffice all the 'hard' options."
            ),
            Self::AsyncInternal { source } => write!(f, "{}", source),
            Self::InvalidTextureFormat(format) => write!(f, "Invalid texture format: {:?}", format),
            Self::RenderContextNotFound => write!(f, "No render contexts are found"),
            Self::WgpuInternal { source } => write!(f, "{}", source),
        }
    }
}

struct TheGpuComputeContext {}

struct TheGpuRenderContext<'w> {
    scale_factor: f32,

    surface: wgpu::Surface<'w>,
    surface_config: wgpu::SurfaceConfiguration,
}

impl<'w> TheGpuRenderContext<'w> {
    fn new(
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        width: u32,
        height: u32,
        scale_factor: f32,
        surface: wgpu::Surface<'w>,
    ) -> Self {
        let capabilities = surface.get_capabilities(adapter);
        let format = capabilities
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(capabilities.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            format,
            width,
            height,
            present_mode: capabilities.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(device, &surface_config);

        Self {
            scale_factor,
            surface,
            surface_config,
        }
    }

    fn draw(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        render_pass: &impl TheRenderPass,
    ) -> Result<wgpu::SurfaceTexture, TheGpuContextError> {
        let surface_texture = self.surface.get_current_texture().map_err(map_wgpu_error)?;

        render_pass.draw(
            device,
            encoder,
            self.scale_factor,
            &surface_texture,
            &self.surface_config,
        );

        Ok(surface_texture)
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if self.surface_config.width == width && self.surface_config.height == height {
            return;
        }

        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(device, &self.surface_config);
    }

    pub fn set_scale_factor(&mut self, scale_factor: f32) {
        self.scale_factor = scale_factor;
    }
}

pub struct TheGpuShaderInfo<'s> {
    pub entry: &'s str,
    pub source: &'s str,
}

impl<'s> TheGpuShaderInfo<'s> {
    pub fn from_source(source: &'s str) -> Self {
        Self {
            entry: "main",
            source,
        }
    }
}

struct TheSurfaceInfo<'w> {
    width: u32,
    height: u32,
    scale_factor: f32,
    surface: wgpu::Surface<'w>,
}

pub struct TheGpuContext<'w> {
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    instance: wgpu::Instance,
    queue: wgpu::Queue,

    compute_context: Option<TheGpuComputeContext>,
    render_context: Option<TheGpuRenderContext<'w>>,
}

impl<'w> TheGpuContext<'w> {
    pub fn create_surface<W>(&self, target: W) -> Result<wgpu::Surface<'w>, TheGpuContextError>
    where
        W: wgpu::WindowHandle + 'w,
    {
        self.instance.create_surface(target).map_err(map_wgpu_error)
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn draw(&self, render_pass: &impl TheRenderPass) -> Result<(), TheGpuContextError> {
        let Some(context) = &self.render_context else {
            return Err(TheGpuContextError::RenderContextNotFound);
        };

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Command Encoder"),
            });

        let surface_texture = context.draw(&self.device, &mut encoder, render_pass)?;

        self.queue.submit(once(encoder.finish()));

        surface_texture.present();

        Ok(())
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if let Some(context) = &mut self.render_context {
            context.resize(&self.device, width, height);
        }
    }

    pub fn set_scale_factor(&mut self, scale_factor: f32) {
        if let Some(context) = &mut self.render_context {
            context.set_scale_factor(scale_factor);
        }
    }

    pub fn set_surface(
        &mut self,
        width: u32,
        height: u32,
        scale_factor: f32,
        surface: wgpu::Surface<'w>,
    ) {
        self.render_context = Some(TheGpuRenderContext::new(
            &self.adapter,
            &self.device,
            width,
            height,
            scale_factor,
            surface,
        ));
    }

    pub fn translate_coord_to_local(&self, x: u32, y: u32) -> (u32, u32) {
        if let Some(context) = &self.render_context {
            (
                (x as f32 / context.scale_factor) as u32,
                (y as f32 / context.scale_factor) as u32,
            )
        } else {
            (x, y)
        }
    }
}

impl<'w> TheGpuContext<'w> {
    pub async fn new() -> Result<Self, TheGpuContextError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        if let Some(adapter) = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
        {
            let (device, queue) = adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        required_features: wgpu::Features::TEXTURE_BINDING_ARRAY,
                        #[cfg(target_arch = "wasm32")]
                        required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                        #[cfg(not(target_arch = "wasm32"))]
                        required_limits: wgpu::Limits::default(),
                        ..Default::default()
                    },
                    None,
                )
                .await
                .map_err(map_wgpu_error)?;

            Ok(Self {
                adapter,
                device,
                instance,
                queue,

                compute_context: None,

                render_context: None,
            })
        } else {
            Err(TheGpuContextError::AdapterNotFound)
        }
    }
}

impl<'w> TheGpuContext<'w> {
    fn capture(&self, texture: &wgpu::Texture) -> Result<Vec<u8>, TheGpuContextError> {
        let Some(context) = &self.render_context else {
            return Err(TheGpuContextError::RenderContextNotFound);
        };

        let width = context.surface_config.width;
        let height = context.surface_config.height;
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Capture Command Encoder"),
            });

        let output_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Capture Texture"),
            size: texture.size(),
            mip_level_count: texture.mip_level_count(),
            sample_count: texture.sample_count(),
            dimension: texture.dimension(),
            format: texture.format(),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let output_texture = output_texture.as_image_copy();
        encoder.copy_texture_to_texture(texture.as_image_copy(), output_texture, size);

        let src = output_texture;

        let align_width =
            align_up(width * 4 * U8_SIZE, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) / U8_SIZE;
        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Capture Buffer"),
            size: (align_width * height) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let dst = wgpu::ImageCopyBuffer {
            buffer: &output_buffer,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(align_width),
                rows_per_image: Some(height),
            },
        };

        encoder.copy_texture_to_buffer(src, dst, size);

        let submission_index = self.queue.submit(Some(encoder.finish()));

        let buffer_slice = output_buffer.slice(..);

        let (sender, receiver) = channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |r| drop(sender.send(r)));

        self.device
            .poll(wgpu::Maintain::WaitForSubmissionIndex(submission_index));
        receiver
            .recv()
            .map_err(map_async_error)?
            .map_err(map_async_error)?;

        let to_rgba = match texture.format() {
            wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb => {
                Ok([0, 1, 2, 3])
            }
            wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb => {
                Ok([2, 1, 0, 3])
            }
            _ => return Err(TheGpuContextError::InvalidTextureFormat(texture.format())),
        }?;

        let mut output = Vec::with_capacity((width * height) as usize * 4);
        for padded_row in buffer_slice.get_mapped_range().chunks(align_width as usize) {
            let row = &padded_row[..width as usize * 4];
            for color in row.chunks(4) {
                output.push(color[to_rgba[0]]);
                output.push(color[to_rgba[1]]);
                output.push(color[to_rgba[2]]);
                output.push(color[to_rgba[3]]);
            }
        }

        output_buffer.unmap();

        Ok(output)
    }
}

fn align_up(num: u32, align: u32) -> u32 {
    (num + align - 1) & !(align - 1)
}

fn map_async_error(err: impl Error + 'static) -> TheGpuContextError {
    TheGpuContextError::AsyncInternal {
        source: Box::new(err),
    }
}

fn map_wgpu_error(err: impl Error + 'static) -> TheGpuContextError {
    TheGpuContextError::WgpuInternal {
        source: Box::new(err),
    }
}
