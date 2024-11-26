use std::{borrow::Cow, sync::mpsc::channel};

use theframework::{prelude::*, wgpu};

const U8_SIZE: u32 = std::mem::size_of::<u8>() as u32;

pub struct Compute {
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::ComputePipeline,
    output_buffer: wgpu::Buffer,
    output_texture: wgpu::Texture,
    texture_size: wgpu::Extent3d,
}

impl TheComputePass for Compute {
    fn compute(
        &self,
        _device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<(), TheGpuContextError> {
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &self.bind_group, &[]);
            compute_pass.dispatch_workgroups(self.width(), self.height(), 1);
        }

        encoder.copy_texture_to_buffer(
            self.output_texture.as_image_copy(),
            wgpu::ImageCopyBuffer {
                buffer: &self.output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(align_width(self.width())),
                    rows_per_image: Some(self.height()),
                },
            },
            self.texture_size,
        );

        Ok(())
    }
}

impl Compute {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let img = image::load_from_memory(include_bytes!("test.png")).unwrap();
        let width = img.width();
        let height = img.height();
        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let buffer = img.into_bytes();

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
        });

        queue.write_texture(
            texture.as_image_copy(),
            &buffer,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            texture_size,
        );

        let output_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Output Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let output_texture_view =
            output_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        view_dimension: wgpu::TextureViewDimension::D2,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        access: wgpu::StorageTextureAccess::WriteOnly,
                    },
                    count: None,
                },
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&output_texture_view),
                },
            ],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader Module"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("compute.wgsl"))),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Buffer"),
            size: (align_width(width) * height) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            bind_group,
            pipeline,
            output_buffer,
            output_texture,
            texture_size,
        }
    }

    pub fn buffer(&self, device: &wgpu::Device) -> Vec<u8> {
        let buffer_slice = self.output_buffer.slice(..);

        let (sender, receiver) = channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |r| drop(sender.send(r)));

        device.poll(wgpu::Maintain::Wait);
        receiver.recv().unwrap().unwrap();

        let mut output = Vec::with_capacity((self.width() * self.height() * 4) as usize);
        for padded_row in buffer_slice
            .get_mapped_range()
            .chunks(align_width(self.width()) as usize)
        {
            let row = &padded_row[..self.width() as usize * 4];
            for color in row.chunks(4) {
                output.push(color[0]);
                output.push(color[1]);
                output.push(color[2]);
                output.push(color[3]);
            }
        }

        self.output_buffer.unmap();

        output
    }

    pub fn height(&self) -> u32 {
        self.texture_size.height
    }

    pub fn width(&self) -> u32 {
        self.texture_size.width
    }
}

fn align_up(num: u32, align: u32) -> u32 {
    (num + align - 1) & !(align - 1)
}

fn align_width(width: u32) -> u32 {
    align_up(width * 4 * U8_SIZE, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) / U8_SIZE
}
