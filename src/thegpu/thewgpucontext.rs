use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    error::Error,
    fmt,
    iter::once,
    num::NonZeroU32,
    sync::mpsc::channel,
};

use aict::{Factory, FactoryBuilder};
use bytemuck::{Pod, Zeroable};
use indexmap::IndexSet;
use wgpu::util::DeviceExt;

use crate::prelude::*;

const U8_SIZE: u32 = std::mem::size_of::<u8>() as u32;

type TheRenderLayerId = usize;
pub type TheTextureId = wgpu::Id<wgpu::Texture>;

#[derive(Debug)]
pub enum TheWgpuContextError {
    AdapterNotFound,
    AsyncInternal { source: Box<dyn Error + 'static> },
    InvalidTextureFormat(wgpu::TextureFormat),
    RenderContextNotFound,
    WgpuInternal { source: Box<dyn Error + 'static> },
}

impl Error for TheWgpuContextError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::AsyncInternal { source } => Some(source.as_ref()),
            Self::WgpuInternal { source } => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl fmt::Display for TheWgpuContextError {
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

struct TheModifyState<T> {
    has_modified: bool,
    value: Option<T>,
}

impl<T> Default for TheModifyState<T> {
    fn default() -> Self {
        Self {
            has_modified: false,
            value: None,
        }
    }
}

impl<T> TheModifyState<T> {
    fn is_not_vacant(&self) -> bool {
        self.value.is_some()
    }

    fn modify(&mut self, value: T) {
        self.has_modified = true;
        self.value = Some(value);
    }

    fn take(&mut self) -> Option<T> {
        self.has_modified = self.value.is_some();
        self.value.take()
    }

    fn take_if_modified(&mut self) -> Option<T> {
        self.has_modified.then(|| self.take())?
    }

    fn use_ref(&mut self) -> Option<&T> {
        self.has_modified = false;
        self.value.as_ref()
    }

    fn use_ref_if_modified(&mut self) -> Option<&T> {
        self.has_modified.then(|| self.use_ref())?
    }
}

struct TheSurfaceInfo<'w> {
    width: u32,
    height: u32,
    scale_factor: f32,
    surface: wgpu::Surface<'w>,
}

struct TheTextureCoordInfo {
    id: TheTextureId,
    coord: Vec2<f32>,
}

impl TheTextureCoordInfo {
    fn vertices(
        &self,
        surface_size: Vec2<f32>,
        layer_coord: Vec2<f32>,
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

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct TheVertexParams {
    transform: [f32; 16],
}

pub struct TheWgpuRenderLayer {
    clip_rect: Option<Vec4<u32>>,
    coord: Vec2<f32>,
    hidden: bool,
    texture_array: Vec<TheTextureCoordInfo>,
    transform: TheTransformMatrix,
    viewport_size: Vec2<u32>,
}

impl TheWgpuRenderLayer {
    pub fn new(viewport_size: Vec2<u32>) -> Self {
        Self {
            clip_rect: None,
            coord: Vec2::zero(),
            hidden: false,
            texture_array: vec![],
            transform: TheTransformMatrix::default(),
            viewport_size,
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
        todo!("Fix rotation skewing");
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

    fn set_viewport_size(&mut self, viewport_size: Vec2<u32>) {
        self.viewport_size = viewport_size;
    }
}

pub struct TheWgpuShaderInfo<'s> {
    pub entry: &'s str,
    pub source: &'s str,
}

impl<'s> TheWgpuShaderInfo<'s> {
    pub fn from_source(source: &'s str) -> Self {
        Self {
            entry: "main",
            source,
        }
    }
}

struct TheWgpuComputeContext {}

struct TheWgpuRenderContext<'w, 's> {
    scale_factor: f32,

    surface: wgpu::Surface<'w>,
    surface_config: wgpu::SurfaceConfiguration,

    sampler: wgpu::Sampler,

    fragment_entry: &'s str,
    fragment_shader: wgpu::ShaderModule,

    vertex_entry: &'s str,
    vertex_shader: wgpu::ShaderModule,
}

impl<'w, 's> TheWgpuRenderContext<'w, 's> {
    fn new(
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        surface_info: TheSurfaceInfo<'w>,
        fragment_entry: &'s str,
        fragment_shader: wgpu::ShaderModule,
        vertex_entry: &'s str,
        vertex_shader: wgpu::ShaderModule,
    ) -> Self {
        let surface = surface_info.surface;
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
            width: surface_info.width,
            height: surface_info.height,
            present_mode: capabilities.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(device, &surface_config);

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

        Self {
            scale_factor: surface_info.scale_factor,
            surface,
            surface_config,

            sampler,

            fragment_entry,
            fragment_shader,

            vertex_entry,
            vertex_shader,
        }
    }

    fn draw(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        texture_map: &HashMap<TheTextureId, TheTextureDataInfo>,
        layers: Vec<&TheWgpuRenderLayer>,
        global_transform: &TheTransformMatrix,
    ) -> Result<wgpu::SurfaceTexture, TheWgpuContextError> {
        let surface_texture = self.surface.get_current_texture().map_err(map_wgpu_error)?;
        let surface_size = Vec2::new(
            self.surface_config.width as f32 / self.scale_factor,
            self.surface_config.height as f32 / self.scale_factor,
        );

        let visible_layers = layers.iter().enumerate().filter(|(_, layer)| !layer.hidden);
        let texture_groups = visible_layers
            .map(|(index, layer)| {
                let textures_and_vertices = layer
                    .texture_array
                    .iter()
                    .filter_map(|texture_coord| {
                        texture_map.get(&texture_coord.id).map(|texture| {
                            let vertices = texture_coord.vertices(
                                surface_size,
                                ndc(layer.coord, surface_size),
                                texture.size,
                            );

                            (&texture.texture, vertices)
                        })
                    })
                    .collect::<Vec<(&wgpu::TextureView, [[f32; 2]; 6])>>();
                let chunked_textures_and_vertices = textures_and_vertices
                    .chunks(device.limits().max_sampled_textures_per_shader_stage as usize)
                    .map(|textures_and_vertices| textures_and_vertices.to_vec());

                chunked_textures_and_vertices
                    .map(move |textures_and_vertices| {
                        let (textures, vertices): (Vec<&wgpu::TextureView>, Vec<[[f32; 2]; 6]>) =
                            textures_and_vertices.into_iter().unzip();

                        (index, (textures, vertices.into_flattened()))
                    })
                    .collect::<Vec<(usize, (Vec<&wgpu::TextureView>, Vec<[f32; 2]>))>>()
            })
            .flatten()
            .collect::<Vec<(usize, (Vec<&wgpu::TextureView>, Vec<[f32; 2]>))>>();

        let view = &surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                format: Some(self.surface_config.format),
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
        for (index, (textures, vertices)) in texture_groups {
            if textures.is_empty() {
                continue;
            }

            let Some(layer) = layers.iter().nth(index) else {
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
                        entry_point: self.vertex_entry,
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        buffers: &[vertex_buffer_layout],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &self.fragment_shader,
                        entry_point: self.fragment_entry,
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: self.surface_config.format,
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
                ndc(
                    global_transform.origin + layer.transform.origin,
                    surface_size,
                ),
                global_transform.rotation + layer.transform.rotation,
                global_transform.scale * layer.transform.scale,
                2.0 * (global_transform.translation + layer.transform.translation) / surface_size,
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
                ],
            });

            render_pass.set_bind_group(0, &bind_group, &[]);
            if let Some(rect) = layer.clip_rect {
                render_pass.set_scissor_rect(rect.x, rect.y, rect.z, rect.w);
            }
            render_pass.draw(0..vertices.len() as u32, 0..texture_count);
        }

        Ok(surface_texture)
    }

    fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if self.surface_config.width == width && self.surface_config.height == height {
            return;
        }

        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(device, &self.surface_config);
    }

    fn set_scale_factor(&mut self, scale_factor: f32) {
        self.scale_factor = scale_factor;
    }

    fn update_fragment_shader(&mut self, entry: &'s str, shader: wgpu::ShaderModule) {
        self.fragment_entry = entry;
        self.fragment_shader = shader;
    }

    fn update_vertex_shader(&mut self, entry: &'s str, shader: wgpu::ShaderModule) {
        self.vertex_entry = entry;
        self.vertex_shader = shader;
    }
}

pub struct TheWgpuContext<'w, 's> {
    layer_group: BTreeMap<isize, IndexSet<TheRenderLayerId>>,
    layer_zindex: HashMap<TheRenderLayerId, isize>,
    layers: HashMap<TheRenderLayerId, TheWgpuRenderLayer>,
    texture_map: HashMap<TheTextureId, TheTextureDataInfo>,
    transform: TheTransformMatrix,

    adapter: wgpu::Adapter,
    device: wgpu::Device,
    instance: wgpu::Instance,
    queue: wgpu::Queue,

    compute_context: Option<TheWgpuComputeContext>,

    render_context: Option<TheWgpuRenderContext<'w, 's>>,
    surface_info: TheModifyState<TheSurfaceInfo<'w>>,
    fragment_shader_info: TheModifyState<TheWgpuShaderInfo<'s>>,
    vertex_shader_info: TheModifyState<TheWgpuShaderInfo<'s>>,

    layer_id_factory: Factory<usize>,
    queue_capture: bool,
}

impl<'w, 's> TheWgpuContext<'w, 's> {
    pub fn add_layer(&mut self) -> TheRenderLayerId {
        let Some(context) = &self.render_context else {
            panic!("No render contexts are found");
        };

        let Ok(id) = self.layer_id_factory.next() else {
            panic!("Cannot retrieve an unique id for the new layer.");
        };

        let layer = TheWgpuRenderLayer::new(Vec2::new(
            context.surface_config.width,
            context.surface_config.height,
        ));

        self.layers.insert(id, layer);
        self.layer_zindex.insert(id, 0);
        self.layer_group.get_mut(&0).unwrap().insert(id);

        id
    }

    pub fn create_surface<W>(&self, target: W) -> Result<wgpu::Surface<'w>, TheWgpuContextError>
    where
        W: wgpu::WindowHandle + 'w,
    {
        self.instance.create_surface(target).map_err(map_wgpu_error)
    }

    pub fn draw(&self) -> Result<Option<Vec<u8>>, TheWgpuContextError> {
        let Some(context) = &self.render_context else {
            return Err(TheWgpuContextError::RenderContextNotFound);
        };

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Command Encoder"),
            });

        let ordered_layers = self
            .layer_group
            .values()
            .map(|set| set.iter())
            .flatten()
            .filter_map(|id| self.layers.get(id))
            .collect::<Vec<&TheWgpuRenderLayer>>();

        let surface_texture = context.draw(
            &self.device,
            &mut encoder,
            &self.texture_map,
            ordered_layers,
            &self.transform,
        )?;

        self.queue.submit(once(encoder.finish()));

        let capture = if self.queue_capture {
            Some(self.capture(&surface_texture.texture)?)
        } else {
            None
        };

        surface_texture.present();

        Ok(capture)
    }

    pub fn layer(&self, layer_id: TheRenderLayerId) -> Option<&TheWgpuRenderLayer> {
        self.layers.get(&layer_id)
    }

    pub fn layer_mut(&mut self, layer_id: TheRenderLayerId) -> Option<&mut TheWgpuRenderLayer> {
        self.layers.get_mut(&layer_id)
    }

    pub fn load_texture(&mut self, width: u32, height: u32, buffer: &[u8]) -> TheTextureId {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
        });

        self.queue.write_texture(
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

    pub fn remove_layer(&mut self, layer_id: TheRenderLayerId) -> Option<TheWgpuRenderLayer> {
        self.layer_id_factory.remove(layer_id);
        if let Some(zindex) = self.layer_zindex.remove(&layer_id) {
            if let Some(set) = self.layer_group.get_mut(&zindex) {
                set.shift_remove(&layer_id);
            }
        }
        self.layers.remove(&layer_id)
    }

    pub fn request_capture(&mut self, capture: bool) {
        self.queue_capture = capture;
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if let Some(context) = &mut self.render_context {
            context.resize(&self.device, width, height);
        }

        for layer in self.layers.values_mut() {
            layer.set_viewport_size(Vec2::new(width, height));
        }
    }

    pub fn rotate(&mut self, theta: f32) {
        self.transform.rotation = theta;
    }

    pub fn scale(&mut self, scale: f32) {
        self.transform.scale = Vec2::new(scale, scale);
    }

    pub fn set_compute_shader(&mut self, shader_info: TheWgpuShaderInfo<'s>) {
        todo!()
    }

    pub fn set_fragment_shader(&mut self, shader: TheWgpuShaderInfo<'s>) {
        self.fragment_shader_info.modify(shader);

        self.try_update_render_context();
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
        self.surface_info.modify(TheSurfaceInfo {
            width,
            height,
            scale_factor,
            surface,
        });

        self.try_update_render_context();

        self.resize(width, height);
    }

    pub fn set_vertex_shader(&mut self, shader: TheWgpuShaderInfo<'s>) {
        self.vertex_shader_info.modify(shader);

        self.try_update_render_context();
    }

    pub fn translate(&mut self, x: f32, y: f32) {
        self.transform.translation = Vec2::new(x, y);
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

    pub fn unload_texture(&mut self, texture_id: TheTextureId) {
        let _ = self.texture_map.remove(&texture_id);
    }
}

impl<'w, 's> TheWgpuContext<'w, 's> {
    pub async fn new() -> Result<Self, TheWgpuContextError> {
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

            let mut layer_group = BTreeMap::new();
            layer_group.insert(0, IndexSet::new());
            let layer_id_factory = FactoryBuilder::new().build();

            Ok(Self {
                layer_group,
                layer_zindex: HashMap::new(),
                layers: HashMap::new(),
                texture_map: HashMap::new(),
                transform: TheTransformMatrix::default(),

                adapter,
                device,
                instance,
                queue,

                compute_context: None,

                render_context: None,
                surface_info: TheModifyState::default(),
                fragment_shader_info: TheModifyState::default(),
                vertex_shader_info: TheModifyState::default(),

                layer_id_factory,
                queue_capture: false,
            })
        } else {
            Err(TheWgpuContextError::AdapterNotFound)
        }
    }

    pub async fn with_default_shaders() -> Result<Self, TheWgpuContextError> {
        let mut context = Self::new().await?;

        context.set_fragment_shader(TheWgpuShaderInfo::from_source(include_str!(
            "./shaders/fragment.wgsl"
        )));
        context.set_vertex_shader(TheWgpuShaderInfo::from_source(include_str!(
            "./shaders/vertex.wgsl"
        )));

        Ok(context)
    }
}

impl<'w, 's> TheWgpuContext<'w, 's> {
    fn capture(&self, texture: &wgpu::Texture) -> Result<Vec<u8>, TheWgpuContextError> {
        let Some(context) = &self.render_context else {
            return Err(TheWgpuContextError::RenderContextNotFound);
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
            _ => return Err(TheWgpuContextError::InvalidTextureFormat(texture.format())),
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

    fn try_update_render_context(&mut self) {
        // Create a new context when surface has changed,
        // and fragment and vertex shaders are ready
        if self.fragment_shader_info.is_not_vacant() && self.vertex_shader_info.is_not_vacant() {
            if let Some(surface_info) = self.surface_info.take_if_modified() {
                let fragment_shader_info = self.fragment_shader_info.use_ref().unwrap();
                let vertex_shader_info = self.vertex_shader_info.use_ref().unwrap();

                self.render_context = Some(TheWgpuRenderContext::new(
                    &self.adapter,
                    &self.device,
                    surface_info,
                    fragment_shader_info.entry,
                    create_shader(&self.device, "Fragment Shader", fragment_shader_info.source),
                    vertex_shader_info.entry,
                    create_shader(&self.device, "Vertex Shader", vertex_shader_info.source),
                ));
                return;
            }
        }

        let Some(context) = &mut self.render_context else {
            return;
        };

        // Update existing context when surface has not changed
        if let Some(shader_info) = self.fragment_shader_info.use_ref_if_modified() {
            context.update_fragment_shader(
                shader_info.entry,
                create_shader(&self.device, "Fragment Shader", shader_info.source),
            );
        }
        if let Some(shader_info) = self.vertex_shader_info.use_ref_if_modified() {
            context.update_vertex_shader(
                shader_info.entry,
                create_shader(&self.device, "Vertex Shader", shader_info.source),
            );
        }
    }
}

fn align_up(num: u32, align: u32) -> u32 {
    (num + align - 1) & !(align - 1)
}

fn create_shader(device: &wgpu::Device, label: &str, source: &str) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(Cow::from(source)),
    })
}

fn map_async_error(err: impl Error + 'static) -> TheWgpuContextError {
    TheWgpuContextError::AsyncInternal {
        source: Box::new(err),
    }
}

fn map_wgpu_error(err: impl Error + 'static) -> TheWgpuContextError {
    TheWgpuContextError::WgpuInternal {
        source: Box::new(err),
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
) -> Mat4<f32> {
    let cos = rotation.cos();
    let sin = rotation.sin();

    let tx = -origin.x * scale.x * cos + origin.y * scale.y * sin + origin.x + translation.x;
    let ty = -origin.x * scale.x * sin - origin.y * scale.y * cos + origin.y - translation.y;

    #[rustfmt::skip]
    let matrix = Mat4::new(
        scale.x * cos, -scale.y * sin, 0.0, 0.0,
        scale.x * sin, scale.y * cos, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        tx, ty, 0.0, 1.0,
    );
    matrix
}
