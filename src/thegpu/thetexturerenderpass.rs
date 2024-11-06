use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    num::NonZeroU32,
};

use aict::{Factory, FactoryBuilder};
use bytemuck::{Pod, Zeroable};
use indexmap::IndexSet;
use itertools::multiunzip;
use wgpu::util::DeviceExt;

use crate::prelude::*;

type TheRenderLayerId = usize;
pub type TheTextureId = wgpu::Id<wgpu::Texture>;

const MAX_TEXTURES_IN_GROUP: usize = 16;

#[repr(C, align(16))]
#[derive(Copy, Clone, Pod, Zeroable)]
struct AlignedVec2 {
    data: [f32; 2],
    _padding: [f32; 2],
}

impl AlignedVec2 {
    pub fn new(data: [f32; 2]) -> Self {
        Self {
            data,
            _padding: [0.0; 2],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct TheFragmentParams {
    bounds: [AlignedVec2; MAX_TEXTURES_IN_GROUP],
    min_coords: [AlignedVec2; MAX_TEXTURES_IN_GROUP],
}

struct TheTransformMatrix {
    origin: Vec2<f32>,
    rotation: f32,
    scale: Vec2<f32>,
    translation: Vec2<f32>,
}

impl Default for TheTransformMatrix {
    fn default() -> Self {
        Self {
            origin: Vec2::zero(),
            rotation: 0.0,
            scale: Vec2::new(1.0, 1.0),
            translation: Vec2::new(0.0, 0.0),
        }
    }
}

struct TheTextureCoordInfo {
    id: TheTextureId,
    coord: Vec2<f32>,
}

impl TheTextureCoordInfo {
    fn vertices(
        &self,
        layer_coord: Vec2<f32>,
        surface_size: Vec2<f32>,
        texture_size: Vec2<f32>,
    ) -> [[f32; 2]; 6] {
        let x = layer_coord.x + 2.0 * self.coord.x / surface_size.x;
        let y = layer_coord.y - 2.0 * self.coord.y / surface_size.y;
        let w = 2.0 * texture_size.x / surface_size.x;
        let h = 2.0 * texture_size.y / surface_size.y;
        [
            [x + w, y],
            [x, y - h],
            [x, y],
            [x + w, y],
            [x, y - h],
            [x + w, y - h],
        ]
    }
}

struct TheTextureDataInfo {
    size: Vec2<f32>,
    texture: wgpu::TextureView,
}

pub struct TheTextureRenderLayer {
    clip_rect: Option<Vec4<u32>>,
    coord: Vec2<f32>,
    hidden: bool,
    texture_array: Vec<TheTextureCoordInfo>,
    transform: TheTransformMatrix,
}

impl TheTextureRenderLayer {
    pub fn new() -> Self {
        Self {
            clip_rect: None,
            coord: Vec2::zero(),
            hidden: false,
            texture_array: vec![],
            transform: TheTransformMatrix::default(),
        }
    }

    pub fn clear(&mut self) {
        self.texture_array.clear();
    }

    pub fn place_texture(&mut self, texture_id: TheTextureId, coord: Vec2<f32>) {
        self.texture_array.push(TheTextureCoordInfo {
            id: texture_id,
            coord,
        })
    }

    pub fn rotate(&mut self, theta: f32) {
        self.transform.rotation = theta;
    }

    pub fn scale(&mut self, scale: f32) {
        self.transform.scale = Vec2::new(scale, scale);
    }

    pub fn set_clip_rect(&mut self, rect: Option<Vec4<u32>>) {
        self.clip_rect = rect;
    }

    pub fn set_origin(&mut self, origin: Vec2<f32>) {
        self.transform.origin = origin
    }

    pub fn set_coord(&mut self, x: f32, y: f32) {
        self.coord = Vec2::new(x, y);
    }

    pub fn translate(&mut self, x: f32, y: f32) {
        self.transform.translation = Vec2::new(x, y);
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct TheVertexParams {
    transform: [f32; 16],
}

pub struct TheTextureRenderPass {
    layer_group: BTreeMap<isize, IndexSet<TheRenderLayerId>>,
    layer_zindex: HashMap<TheRenderLayerId, isize>,
    layers: HashMap<TheRenderLayerId, TheTextureRenderLayer>,
    texture_map: HashMap<TheTextureId, TheTextureDataInfo>,
    transform: TheTransformMatrix,

    layer_id_factory: Factory<usize>,

    fragment_shader: wgpu::ShaderModule,
    vertex_shader: wgpu::ShaderModule,

    sampler: wgpu::Sampler,
}

impl TheRenderPass for TheTextureRenderPass {
    fn draw(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        scale_factor: f32,
        surface_texture: &wgpu::SurfaceTexture,
        surface_config: &wgpu::SurfaceConfiguration,
    ) -> Result<(), TheGpuContextError> {
        let surface_size = Vec2::new(
            surface_config.width as f32 / scale_factor,
            surface_config.height as f32 / scale_factor,
        );

        let ordered_layers = self
            .layer_group
            .values()
            .flatten()
            .filter_map(|id| self.layers.get(id))
            .collect::<Vec<&TheTextureRenderLayer>>();

        let chunk_size = device
            .limits()
            .max_sampled_textures_per_shader_stage
            .min(device.limits().max_storage_buffers_per_shader_stage)
            as usize;
        let chunk_size = chunk_size.max(MAX_TEXTURES_IN_GROUP);
        let texture_groups = ordered_layers
            .iter()
            .enumerate()
            .filter(|(_, layer)| !layer.hidden)
            .map(|(index, layer)| {
                let textures_vertices_bounds = layer
                    .texture_array
                    .iter()
                    .filter_map(|texture_coord| {
                        self.texture_map.get(&texture_coord.id).map(|texture| {
                            let vertices = texture_coord.vertices(
                                ndc(layer.coord, surface_size),
                                surface_size,
                                texture.size,
                            );
                            let bounds = texture.size / surface_size;
                            let bounds = [bounds[0], bounds[1]];
                            let min_coord = [vertices[2][0], vertices[2][1]];

                            (&texture.texture, vertices.to_vec(), bounds, min_coord)
                        })
                    })
                    .collect::<Vec<(&wgpu::TextureView, Vec<[f32; 2]>, [f32; 2], [f32; 2])>>();

                textures_vertices_bounds
                    .chunks(chunk_size)
                    .map(|textures_vertices_bounds| {
                        let (textures, vertices, bounds, min_coords): (
                            Vec<&wgpu::TextureView>,
                            Vec<Vec<[f32; 2]>>,
                            Vec<[f32; 2]>,
                            Vec<[f32; 2]>,
                        ) = multiunzip(textures_vertices_bounds.to_vec());

                        (
                            index,
                            (
                                textures,
                                vertices.into_iter().flatten().collect::<Vec<[f32; 2]>>(),
                                bounds,
                                min_coords,
                            ),
                        )
                    })
                    .collect::<Vec<(
                        usize,
                        (
                            Vec<&wgpu::TextureView>,
                            Vec<[f32; 2]>,
                            Vec<[f32; 2]>,
                            Vec<[f32; 2]>,
                        ),
                    )>>()
            })
            .flatten()
            .collect::<Vec<(
                usize,
                (
                    Vec<&wgpu::TextureView>,
                    Vec<[f32; 2]>,
                    Vec<[f32; 2]>,
                    Vec<[f32; 2]>,
                ),
            )>>();

        let view = &surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                format: Some(surface_config.format),
                ..Default::default()
            });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        let mut bind_group_layout = None;
        let mut prev_group_texture_count = 0;
        for (index, (textures, vertices, bounds, min_coords)) in texture_groups {
            if textures.is_empty() {
                continue;
            }

            let Some(layer) = ordered_layers.iter().nth(index) else {
                continue;
            };

            let vertex_data_slice = bytemuck::cast_slice(&vertices);
            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: vertex_data_slice,
                usage: wgpu::BufferUsages::VERTEX,
            });
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));

            let texture_count = textures.len() as u32;
            if prev_group_texture_count != texture_count {
                bind_group_layout = Some(device.create_bind_group_layout(
                    &wgpu::BindGroupLayoutDescriptor {
                        label: Some("Bind Group Layout"),
                        entries: &[
                            wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::VERTEX,
                                ty: wgpu::BindingType::Buffer {
                                    ty: wgpu::BufferBindingType::Uniform,
                                    has_dynamic_offset: false,
                                    min_binding_size: None,
                                },
                                count: None,
                            },
                            wgpu::BindGroupLayoutEntry {
                                binding: 1,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Texture {
                                    multisampled: false,
                                    view_dimension: wgpu::TextureViewDimension::D2,
                                    sample_type: wgpu::TextureSampleType::Float {
                                        filterable: true,
                                    },
                                },
                                count: NonZeroU32::new(texture_count),
                            },
                            wgpu::BindGroupLayoutEntry {
                                binding: 2,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                                count: None,
                            },
                            wgpu::BindGroupLayoutEntry {
                                binding: 3,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Buffer {
                                    ty: wgpu::BufferBindingType::Uniform,
                                    has_dynamic_offset: false,
                                    min_binding_size: None,
                                },
                                count: None,
                            },
                        ],
                    },
                ));

                let pipeline_layout =
                    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("Render Pipeline Layout"),
                        bind_group_layouts: &[bind_group_layout.as_ref().unwrap()],
                        ..Default::default()
                    });

                let vertex_buffer_layout = wgpu::VertexBufferLayout {
                    array_stride: (vertex_data_slice.len() / vertices.len()) as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
                        offset: 0,
                        shader_location: 0,
                    }],
                };

                let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &self.vertex_shader,
                        entry_point: "main",
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        buffers: &[vertex_buffer_layout],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &self.fragment_shader,
                        entry_point: "main",
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: surface_config.format,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    primitive: wgpu::PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                    cache: None,
                });

                render_pass.set_pipeline(&pipeline);

                prev_group_texture_count = texture_count;
            }

            let matrix = transform_matrix(
                ndc(self.transform.origin + layer.transform.origin, surface_size),
                self.transform.rotation + layer.transform.rotation,
                self.transform.scale * layer.transform.scale,
                2.0 * (self.transform.translation + layer.transform.translation) / surface_size,
                surface_size.x / surface_size.y,
            );
            let mut transform = [0.0; 16];
            transform.copy_from_slice(matrix.as_ref());

            let vertex_params = TheVertexParams { transform };
            let vertex_params_buffer =
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Params Buffer"),
                    contents: bytemuck::cast_slice(&[vertex_params]),
                    usage: wgpu::BufferUsages::UNIFORM,
                });

            let mut fragment_params = TheFragmentParams {
                bounds: [AlignedVec2::new([0.0; 2]); MAX_TEXTURES_IN_GROUP],
                min_coords: [AlignedVec2::new([0.0; 2]); MAX_TEXTURES_IN_GROUP],
            };
            for i in 0..texture_count as usize {
                fragment_params.bounds[i].data = bounds[i];
                fragment_params.min_coords[i].data = min_coords[i];
            }
            let fragment_params_buffer =
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Fragment Params Buffer"),
                    contents: bytemuck::cast_slice(&[fragment_params]),
                    usage: wgpu::BufferUsages::UNIFORM,
                });

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Bind Group"),
                layout: bind_group_layout.as_ref().unwrap(),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: vertex_params_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureViewArray(&textures),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: fragment_params_buffer.as_entire_binding(),
                    },
                ],
            });

            render_pass.set_bind_group(0, &bind_group, &[]);
            if let Some(rect) = layer.clip_rect {
                render_pass.set_scissor_rect(rect.x, rect.y, rect.z, rect.w);
            }
            render_pass.draw(0..vertices.len() as u32, 0..texture_count);
        }

        Ok(())
    }
}

impl TheTextureRenderPass {
    pub fn new(device: &wgpu::Device) -> Self {
        let mut layer_group = BTreeMap::new();
        layer_group.insert(0, IndexSet::new());
        let layer_id_factory = FactoryBuilder::new().build();

        let fragment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Fragment Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::from(include_str!("shaders/fragment.wgsl"))),
        });
        let vertex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Vertex Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::from(include_str!("shaders/vertex.wgsl"))),
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

        Self {
            layer_group,
            layer_zindex: HashMap::new(),
            layers: HashMap::new(),
            texture_map: HashMap::new(),
            transform: TheTransformMatrix::default(),

            layer_id_factory,

            fragment_shader,
            vertex_shader,

            sampler,
        }
    }

    pub fn add_layer(&mut self) -> TheRenderLayerId {
        let Ok(id) = self.layer_id_factory.next() else {
            panic!("Cannot retrieve an unique id for the new layer.");
        };

        self.layers.insert(id, TheTextureRenderLayer::new());
        self.layer_zindex.insert(id, 0);
        self.layer_group.get_mut(&0).unwrap().insert(id);

        id
    }

    pub fn layer(&self, layer_id: TheRenderLayerId) -> Option<&TheTextureRenderLayer> {
        self.layers.get(&layer_id)
    }

    pub fn layer_mut(&mut self, layer_id: TheRenderLayerId) -> Option<&mut TheTextureRenderLayer> {
        self.layers.get_mut(&layer_id)
    }

    pub fn load_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        buffer: &[u8],
    ) -> TheTextureId {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
        });

        queue.write_texture(
            texture.as_image_copy(),
            buffer,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        let id = texture.global_id();

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Texture View"),
            ..Default::default()
        });

        self.texture_map.insert(
            id,
            TheTextureDataInfo {
                size: Vec2::new(width as f32, height as f32),
                texture: texture_view,
            },
        );

        id
    }

    pub fn place_texture(
        &mut self,
        layer_id: TheRenderLayerId,
        texture_id: TheTextureId,
        coord: Vec2<f32>,
    ) {
        if let Some(layer) = self.layers.get_mut(&layer_id) {
            layer.place_texture(texture_id, coord);
        }
    }

    pub fn remove_layer(&mut self, layer_id: TheRenderLayerId) -> Option<TheTextureRenderLayer> {
        self.layer_id_factory.remove(layer_id);
        if let Some(zindex) = self.layer_zindex.remove(&layer_id) {
            if let Some(set) = self.layer_group.get_mut(&zindex) {
                set.shift_remove(&layer_id);
            }
        }
        self.layers.remove(&layer_id)
    }

    pub fn rotate(&mut self, theta: f32) {
        self.transform.rotation = theta;
    }

    pub fn scale(&mut self, scale: f32) {
        self.transform.scale = Vec2::new(scale, scale);
    }

    pub fn set_layer_zindex(&mut self, layer_id: TheRenderLayerId, zindex: isize) {
        if let Some(prev_zindex) = self.layer_zindex.get(&layer_id) {
            if let Some(set) = self.layer_group.get_mut(&prev_zindex) {
                set.shift_remove(&layer_id);
            }
        }

        self.layer_zindex.insert(layer_id, zindex);

        if !self.layer_group.contains_key(&zindex) {
            self.layer_group.insert(zindex, IndexSet::new());
        }
        self.layer_group.get_mut(&zindex).unwrap().insert(layer_id);
    }

    pub fn set_origin(&mut self, origin: Vec2<f32>) {
        self.transform.origin = origin;
    }

    pub fn translate(&mut self, x: f32, y: f32) {
        self.transform.translation = Vec2::new(x, y);
    }

    pub fn unload_texture(&mut self, texture_id: TheTextureId) {
        let _ = self.texture_map.remove(&texture_id);
    }
}

fn ndc(coord: Vec2<f32>, device_size: Vec2<f32>) -> Vec2<f32> {
    Vec2::new(
        2.0 * coord.x / device_size.x - 1.0,
        1.0 - 2.0 * coord.y / device_size.y,
    )
}

fn transform_matrix(
    origin: Vec2<f32>,
    rotation: f32,
    scale: Vec2<f32>,
    translation: Vec2<f32>,
    aspect_ratio: f32,
) -> Mat4<f32> {
    #[rustfmt::skip]
    let aspect_correction = Mat4::new(
        1.0 / aspect_ratio, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        origin.x * (1.0 - 1.0 / aspect_ratio), 0.0, 0.0, 1.0,
    );

    #[rustfmt::skip]
    let reverse_aspect_correction = Mat4::new(
        aspect_ratio, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        origin.x * (1.0 - aspect_ratio), 0.0, 0.0, 1.0,
    );

    let rotate = Mat4::from_z_rotation(rotation);
    let scale = Mat4::from_scale(Vec3::new(scale.x, scale.y, 1.0));
    let translate_origin = Mat4::from_translation(Vec3::new(-origin.x, -origin.y, 0.0));
    let translate_back = Mat4::from_translation(Vec3::new(origin.x, origin.y, 0.0));
    let translate = Mat4::from_translation(Vec3::new(translation.x, -translation.y, 0.0));

    reverse_aspect_correction
        * translate_origin.transpose()
        * scale
        * rotate
        * translate_back.transpose()
        * aspect_correction
        * translate.transpose()
}
