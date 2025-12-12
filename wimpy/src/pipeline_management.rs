use std::u8;

use image::{DynamicImage, EncodableLayout, GenericImageView};
use wgpu::{BindGroup, BindGroupLayout, BindGroupLayoutDescriptor, Buffer, BufferUsages, IndexFormat, RenderPipeline, TextureUsages, TextureView, util::{BufferInitDescriptor, DeviceExt}};
use crate::{frame::{FilterMode, WrapMode}, frame_binder::WGPUInterface};

pub struct PipelineManager {
    pipeline: RenderPipeline,
    index_buffer: Buffer,
    vertex_buffer: Buffer,
    view_projection_buffer: Buffer,
    view_projection_bind_group: BindGroup
}

const INDICES_PER_QUAD: u32 = 5;

impl PipelineManager {
    pub fn create(wgpu_interface: &impl WGPUInterface,max_quads: u32) -> Self {

        /* Range checking for u16 limits */
        let (vertex_count,index_count) = {
            const VERTEXES_IN_QUAD: u32 = 4;
            let max = u16::MAX as u32 - 1; /* -1 to adjust for triangle strip primitive restart */
            let vertex_count_estimation = max_quads * VERTEXES_IN_QUAD;
            match vertex_count_estimation > max {
                true => {
                    let vertex_count = (max - VERTEXES_IN_QUAD) / VERTEXES_IN_QUAD;
                    let index_count = vertex_count * VERTEXES_IN_QUAD * INDICES_PER_QUAD;
                    log::warn!("Max quads exceeds u16 indexing limit; truncating request buffer capacity.");
                    (vertex_count,index_count)
                },
                false => (vertex_count_estimation,max_quads * INDICES_PER_QUAD),
            }
        };

        let device = wgpu_interface.get_device();
        let pipeline = create_pipeline(wgpu_interface);

        let view_projection_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("View Projection Buffer"),
            contents: bytemuck::cast_slice(&ViewProjectionMatrix::default()),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let view_projection_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &pipeline.get_bind_group_layout(VIEW_PROJECTION_BIND_GROUP_INDEX),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: view_projection_buffer.as_entire_binding(),
            }],
            label: Some("View Projection Bind Group"),
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor{
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&get_index_buffer_contents(max_quads)),
            usage: wgpu::BufferUsages::INDEX
        });

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor{
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&get_vertex_buffer_contents(max_quads)),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST
        });

        return Self {
            view_projection_buffer,
            view_projection_bind_group,
            pipeline,
            vertex_buffer,
            index_buffer
        };
    }

    pub fn get_texture_bind_group_layout(&self) -> BindGroupLayout {
        return self.pipeline.get_bind_group_layout(TEXTURE_BIND_GROUP_INDEX);
    }
}

fn get_index_buffer_contents(index_count: u32) -> Vec<u16> {
    let mut indices: Vec<u16> = Vec::with_capacity(index_count as usize);
/*
  Triangle strip should generate 0-1-3 1-3-2 (CCW)

                    0---2
                    |  /|
                    | / |
                    |/  |
                    1---3
*/
    let length = index_count as u16;

    for i in 0..length {
        let index = i * 4;
        indices.push(index + 0);
        indices.push(index + 1);
        indices.push(index + 3);
        indices.push(index + 2);
        indices.push(u16::MAX);
    }

    return indices;
}

fn get_vertex_buffer_contents(vertex_count: u32) -> Vec<Vertex> {
    let size = vertex_count as usize;
    let mut vertices: Vec<Vertex> = Vec::with_capacity(size);
    vertices.resize_with(size,Default::default);
    return vertices;
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

pub struct TextureContainer {
    width: u32,
    height: u32,
    texture_view: TextureView,
    bind_groups: Vec<BindGroup>
}

impl TextureContainer {
    pub fn size(&self) -> (u32,u32) {
        return (self.width,self.height);
    }
}

pub const TEXTURE_BIND_GROUP_INDEX: u32 = 0;
pub const VIEW_PROJECTION_BIND_GROUP_INDEX: u32 = 1;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct Vertex {
    pub position: [f32;2],
    pub uv: [f32;2],
    pub color: [f32;4],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute;3] = wgpu::vertex_attr_array![
        0 => Float32x2, //Position
        1 => Float32x3, //UV
        2 => Float32x4, //Color
    ];

    pub fn get_buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        return wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress * 4,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub fn create_pipeline(wgpu_interface: &impl WGPUInterface) -> RenderPipeline {

    let device = wgpu_interface.get_device();

    let texture_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("Texture Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false, /* Must remain false to use STORAGE_BINDING texture usage */
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
        source: wgpu::ShaderSource::Wgsl(include_str!("../../content/shaders/position_uv_color.wgsl").into())
    });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[
            &texture_bind_group_layout, //TEXTURE_BIND_GROUP_INDEX must = 0
            &view_projection_bind_group_layout, //VIEW_PROJECTION_BIND_GROUP_INDEX must = 1
        ],
        push_constant_ranges: &[]
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                format: wgpu_interface.get_output_format(),
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })]
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleStrip,
            strip_index_format: Some(IndexFormat::Uint16),
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
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

    return pipeline;
}

pub enum SamplerMode {
    NearestClamp = 0,
    NearestRepeat = 1,
    NearestMirrorRepeat = 2,
    LinearClamp = 3,
    LinearRepeat = 4,
    LinearMirrorRepeat = 5
}

impl SamplerMode {
    pub fn get_mode(filter_mode: FilterMode,wrap_mode: WrapMode) -> SamplerMode {
        return match (filter_mode,wrap_mode) {
            (FilterMode::Nearest,WrapMode::Clamp) => SamplerMode::NearestClamp,
            (FilterMode::Nearest,WrapMode::Repeat) => SamplerMode::NearestRepeat,
            (FilterMode::Nearest,WrapMode::MirrorRepeat) => SamplerMode::NearestMirrorRepeat,
            (FilterMode::Linear,WrapMode::Clamp) => SamplerMode::LinearClamp,
            (FilterMode::Linear,WrapMode::Repeat) => SamplerMode::LinearRepeat,
            (FilterMode::Linear,WrapMode::MirrorRepeat) => SamplerMode::LinearMirrorRepeat,
        };
    }
}

fn get_bind_groups(texture_view: &TextureView,wgpu_interface: &impl WGPUInterface) -> Vec<BindGroup> {
    let device = wgpu_interface.get_device();

    let bind_group_layout = &wgpu_interface
        .get_pipeline_manager()
        .get_texture_bind_group_layout();

    use wgpu::{FilterMode,AddressMode};
    let bind_groups: Vec<BindGroup> = vec![
        /* Must match index order of SamplerMode */
        (FilterMode::Nearest,AddressMode::ClampToEdge),
        (FilterMode::Nearest,AddressMode::Repeat),
        (FilterMode::Nearest,AddressMode::MirrorRepeat),
        (FilterMode::Linear,AddressMode::ClampToEdge),
        (FilterMode::Linear,AddressMode::Repeat),
        (FilterMode::Linear,AddressMode::MirrorRepeat),
    ].iter().map(|(filter,address)|{
        let (a,f) = (*address,*filter);
        device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: a,
            address_mode_v: a,
            address_mode_w: a,
            mag_filter: f,
            min_filter: f,
            mipmap_filter: f,
            ..Default::default()
        })
    }).map(|sampler|device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            }
        ],
        label: None, //TODO: Make meaningful label
    })).collect();
    return bind_groups;
}

fn validate_dimensions(dimensions: (u32,u32)) {
    if dimensions.0 > 0 && dimensions.1 > 0 {
        return;
    }

    //TODO: Validate max dimension size with WGPU capabilities

    panic!("Invalid texture container dimensions. Dimensions must be greater than 0.");
}

struct TextureCreationParameters {
    dimensions: (u32,u32),
    mutable: bool
}

fn create_texture(
    wgpu_interface: &impl WGPUInterface,
    image_data: Option<&[u8]>,
    parameters: TextureCreationParameters
) -> TextureContainer {

    let dimensions = parameters.dimensions;

    validate_dimensions(dimensions);

    let texture_size = wgpu::Extent3d {
        width: dimensions.0,
        height: dimensions.1,
        depth_or_array_layers: 1,
    };

    let mut usage_flags = TextureUsages::TEXTURE_BINDING;

    if image_data.is_some() {
        usage_flags |= TextureUsages::COPY_DST;
    }

    if parameters.mutable {
        usage_flags |= TextureUsages::RENDER_ATTACHMENT;
    }

    let device = wgpu_interface.get_device();
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,

        //Might want to make this sRGB. No idea what the fuck is going on behind the scenes with this
        format: wgpu::TextureFormat::Rgba8Unorm,

        usage: usage_flags,
        label: None, //TODO: Add useful labels
        view_formats: &[],
    });

    if let Some(data) = image_data {
        wgpu_interface.get_queue().write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                /* 1 byte per color in 8bit 4 channel color (RGBA with u8) */
                bytes_per_row: Some(4*dimensions.0), 
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );
    }

    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let bind_groups = get_bind_groups(&texture_view, wgpu_interface);

    return TextureContainer {
        width: dimensions.0,
        height: dimensions.1,
        texture_view,
        bind_groups
    };
}

impl TextureContainer {
    pub fn create_mutable(dimensions: (u32,u32),wgpu_interface: &impl WGPUInterface) -> TextureContainer {
        return create_texture(wgpu_interface,None,TextureCreationParameters {
            dimensions,
            mutable: true
        });
    }

    pub fn from_image(image: &DynamicImage,wgpu_interface: &impl WGPUInterface) -> TextureContainer {
        let dimensions = image.dimensions();

        //TODO: Make sure alpha channel is premultiplied ... Somehow.
        let image_data = image.to_rgba8();

        return create_texture(wgpu_interface,Some(image_data.as_bytes()),TextureCreationParameters {
            dimensions,
            mutable: false
        });
    }
}
