use std::{collections::HashMap};
use std::sync::Arc;

use winit::window::Window;
use wgpu::{BindGroup, BindGroupLayoutDescriptor, BufferAddress, CommandEncoder, RenderPass, RenderPipeline, TextureView};

pub struct Graphics {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    render_pipelines: HashMap<PipelineVariant,RenderPipeline>,
    /* Look into slotmap */
    bind_groups: HashMap<u32,NamedBindGroup>,
    bind_group_identifiers: HashMap<String,BindGroupReference>,
    /* Find a way to reuse expired counters */
    bind_group_counter: u32
}

struct NamedBindGroup {
    value: BindGroup,
    name: String
}

#[derive(Debug,PartialEq,Eq,Clone,Copy)]
pub enum BindGroupType {
    Texture,
    CameraUniform
}

#[derive(Clone,Copy)]
pub struct BindGroupReference {
    pub bind_group_type: BindGroupType,
    pub id: u32
}

pub const TEXTURE_BIND_GROUP_INDEX: u32 = 0;
pub const CAMERA_UNIFORM_BIND_GROUP_INDEX: u32 = 1;


#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32;2],
    pub color: [f32;3],
    pub uv: [f32;2]
}

impl Vertex {
    pub const SIZE: u32 = size_of::<Vertex>() as u32;

    const POSITION_SIZE: BufferAddress = size_of::<[f32;2]>() as BufferAddress;
    const COLOR_SIZE: BufferAddress = size_of::<[f32;3]>() as BufferAddress;
    const TEXTURE_SIZE: BufferAddress = size_of::<[f32;2]>() as BufferAddress;

    const fn get_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        return wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: Self::POSITION_SIZE,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                    wgpu::VertexAttribute {
                    offset: Self::POSITION_SIZE + Self::COLOR_SIZE,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                }
            ]
        };
    }
}

#[derive(Eq, Hash, PartialEq)]
pub enum PipelineVariant {
    Basic
}

impl Graphics {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Graphics> {

        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window)?;

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptionsBase {
            power_preference: wgpu::PowerPreference::None,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface)
        }).await?;

        let (device,queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            required_limits: wgpu::Limits::default(),
            memory_hints: Default::default(),
            trace: wgpu::Trace::Off
        }).await?;

        let surface_capabilities = surface.get_capabilities(&adapter);

        let surface_format = surface_capabilities.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 2
        };

        let mut render_pipelines = HashMap::new();

        let basic_render_pipeline = create_basic_pipeline(&device,config.format);
        render_pipelines.insert(PipelineVariant::Basic,basic_render_pipeline);

        let bind_groups = HashMap::new();
        let bind_group_identifiers = HashMap::new();

        return Ok(Self {
            surface, device, queue, config, render_pipelines, bind_groups, bind_group_identifiers, bind_group_counter: 0, 
        });
    }

    pub fn get_bind_group(&self,bind_group_reference: &BindGroupReference) -> &BindGroup {
        if let Some(bind_group) = self.bind_groups.get(&bind_group_reference.id) {
            return &bind_group.value;
        } else {
            panic!("Bind group not found!");
        }
    }

    pub fn set_pipeline(&self,render_pass: &mut RenderPass,pipeline_variant: PipelineVariant) {
        if let Some(render_pipeline) = self.render_pipelines.get(&pipeline_variant) {
            render_pass.set_pipeline(render_pipeline);
        } else {
            panic!("Render pipeline type is not implemented.");
        }
    }

    pub fn destroy_bind_group(&mut self,bind_group_reference: BindGroupReference) {
        if let Some(named_bind_group) = self.bind_groups.remove(&bind_group_reference.id) {
            self.bind_group_identifiers.remove(&named_bind_group.name);
        } else {
            panic!("Bind group reference not found!");
        }
    }

    /* Gets a texture reference and loads the texture if it doesn't already exist. */
    pub fn create_texture(&mut self,identifier: &str) -> BindGroupReference {

        if let Some(bind_group_reference) = self.bind_group_identifiers.get(identifier) {
            assert_eq!(
                bind_group_reference.bind_group_type,
                BindGroupType::Texture,
                "Bind group for identifier '{}' is not of the texture type.",
                identifier
            );
            return bind_group_reference.clone();
        }

        let diffuse_bytes = include_bytes!("../../test_image.png");
        let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
        let diffuse_rgba = diffuse_image.to_rgba8();

        use image::GenericImageView;
        let dimensions = diffuse_image.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some(&format!("texture#{}",identifier)),
            view_formats: &[],
        });

        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &diffuse_rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );
        
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let texture_sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {

            layout: &self.render_pipelines[
                &PipelineVariant::Basic
            ].get_bind_group_layout(TEXTURE_BIND_GROUP_INDEX),

            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture_sampler),
                }
            ],
            label: Some(&format!("texture_bind_group#{}",identifier)),
        });


        let id = self.bind_group_counter;
        self.bind_group_counter += 1;

        self.bind_groups.insert(id,NamedBindGroup { value: bind_group, name: identifier.to_string() });

        let bind_group_reference = BindGroupReference { bind_group_type: BindGroupType::Texture, id };
        self.bind_group_identifiers.insert(identifier.to_string(),bind_group_reference);

        return bind_group_reference;
    }
}

pub fn get_basic_render_pass<'encoder>(encoder: &'encoder mut CommandEncoder,view: &'encoder TextureView,clear_color: wgpu::Color) -> RenderPass<'encoder> {
    return encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(clear_color),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        occlusion_query_set: None,
        timestamp_writes: None,
    });
}

fn create_basic_pipeline(device: &wgpu::Device,fragment_format: wgpu::TextureFormat) -> wgpu::RenderPipeline {        
    let texture_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("Texture Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float {
                        filterable: true
                    },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ]
    });

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into())
    });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[&texture_bind_group_layout],
        push_constant_ranges: &[]
    });

    return device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[Vertex::get_buffer_layout()]
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: fragment_format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })]
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None
    });
}
