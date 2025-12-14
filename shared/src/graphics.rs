use std::sync::Arc;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use winit::window::Window;
use wgpu::{
    BindGroup,
    BindGroupLayoutDescriptor,
    Buffer,
    BufferAddress,
    BufferUsages,
    CommandEncoder,
    CommandEncoderDescriptor,
    RenderPass,
    RenderPipeline,
    SurfaceError,
    SurfaceTexture,
    TextureView
};
use collections::named_cache::{CacheItemReference, NamedCache};

pub struct Graphics {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    view_projection_reference: CacheItemReference,
    view_projection_buffer: wgpu::Buffer,
    bind_groups: NamedCache<TypedBindGroup>,
}

pub struct TypedBindGroup {
    value: BindGroup,
    variant: BindGroupVariant
}
#[derive(Debug,PartialEq,Eq,Clone,Copy)]
pub enum BindGroupVariant {
    Texture,
    ViewProjection
}

pub type ViewProjectionMatrix = [[f32;4];4];

#[repr(C)]
#[derive(Debug,Copy,Clone,bytemuck::Pod,bytemuck::Zeroable)]
pub struct ViewProjection {
    value: ViewProjectionMatrix,
}

impl ViewProjection {
    pub fn create(matrix: ViewProjectionMatrix) -> Self {
        return ViewProjection {
            value: matrix
        }
    }
    pub fn get_bytes(&self) -> &[u8] {
        return bytemuck::cast_slice(&self.value);
    }
}

pub const TEXTURE_BIND_GROUP_INDEX: u32 = 0;
pub const VIEW_PROJECTION_BIND_GROUP_INDEX: u32 = 1;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32;2],
    pub uv: [f32;2],
    pub color: [f32;3],
}

impl Vertex {
    pub const SIZE: u32 = size_of::<Vertex>() as u32;
    
    const POSITION_SIZE: BufferAddress = size_of::<[f32;2]>() as BufferAddress;
    const COLOR_SIZE: BufferAddress = size_of::<[f32;3]>() as BufferAddress;

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

fn create_bind_group_name(identifier: &str,bind_group_type: BindGroupVariant) -> String {
    let mut string = String::with_capacity(identifier.len() + 4);
    string.push_str(identifier);
    /* Must be equal length and match the capacity of the allocated String. */
    string.push_str(match bind_group_type {
        BindGroupVariant::Texture => "#tex",
        BindGroupVariant::ViewProjection => "#vwp",
    });
    return string;
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

        let render_pipeline = create_basic_pipeline(&device,config.format);

        let view_projection_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("View Projection Buffer"),
            contents: bytemuck::cast_slice(&ViewProjectionMatrix::default()),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let identifier = create_bind_group_name("0",BindGroupVariant::ViewProjection);

        let view_projection_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &render_pipeline.get_bind_group_layout(VIEW_PROJECTION_BIND_GROUP_INDEX),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: view_projection_buffer.as_entire_binding(),
            }],
            label: Some("View Projection Bind Group"),
        });

        let mut bind_groups = NamedCache::default();

        let view_projection_reference = bind_groups.store_item(&identifier,TypedBindGroup {
            value: view_projection_bind_group,
            variant: BindGroupVariant::ViewProjection
        });

        return Ok(Self {
            surface,
            device,
            queue,
            config,
            render_pipeline,
            bind_groups,
            view_projection_reference,
            view_projection_buffer
        });
    }

    pub fn get_bind_group(&self,bind_group_reference: &CacheItemReference) -> &BindGroup {
        return &self.bind_groups.borrow_item(&bind_group_reference).value;
    }

    pub fn configure_surface_size(&mut self,width: u32,height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device,&self.config);
    }

    pub fn get_default_pipeline(&self) -> &RenderPipeline {
        return &self.render_pipeline;
    }

    pub fn submit_encoder(&self,encoder: CommandEncoder) {  
        self.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn create_buffer(&self,descriptor: &BufferInitDescriptor) -> Buffer {
        return self.device.create_buffer_init(descriptor);
    }

    pub fn drop_bind_group(&mut self,bind_group_reference: &CacheItemReference) {
        self.bind_groups.remove_item(bind_group_reference).value;
    }

    pub fn get_window_surface(&self) -> Result<SurfaceTexture,SurfaceError> {
        return self.surface.get_current_texture();
    }

    pub fn create_command_encoder(&self) -> CommandEncoder {
        return self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder")
        });
    }

    pub fn write_view_projection(&self,view_projection: &Box<ViewProjection>) -> &BindGroup {
        let data = view_projection.get_bytes();
        self.queue.write_buffer(&self.view_projection_buffer,0,data);
        return &self.bind_groups.borrow_item(&self.view_projection_reference).value;
    }

    /* Gets a texture reference and loads the texture if it doesn't already exist. */
    pub fn get_texture(&mut self,identifier: &str) -> CacheItemReference {

        if let Some(bind_group_reference) = self.bind_groups.get_reference(identifier) {
            let bind_group = self.bind_groups.borrow_item(&bind_group_reference);
            assert_eq!(
                bind_group.variant,
                BindGroupVariant::Texture,
                "Bind group for identifier '{}' is not of the texture type.",
                identifier
            );
            return bind_group_reference;
        }

        //TODO: Load from real filesystem
        let diffuse_bytes = include_bytes!("../../content/images/test_image.png");
        let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
        let diffuse_rgba = diffuse_image.to_rgba8();

        use image::GenericImageView;
        let dimensions = diffuse_image.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let bind_group_name = create_bind_group_name(identifier,BindGroupVariant::Texture);

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some(&bind_group_name),
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
        
        //TODO Texture sampler customization
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let texture_sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.render_pipeline.get_bind_group_layout(TEXTURE_BIND_GROUP_INDEX),
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
            label: Some(&bind_group_name),
        });

        return self.bind_groups.store_item(&bind_group_name,TypedBindGroup {
            value: bind_group,
            variant: BindGroupVariant::Texture,
        });
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

    let view_projection_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
        label: Some("View Projection Bind Group Layout"),
    });

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../../content/shaders/quads.wgsl").into())
    });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[
            &texture_bind_group_layout,
            &view_projection_bind_group_layout,
        ],
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
