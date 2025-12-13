use bytemuck::{
    Pod,
    Zeroable
};
use image::{
    DynamicImage,
    EncodableLayout,
    GenericImageView
};

use wgpu::{
    BindGroup,
    BindGroupLayout,
    BindGroupLayoutDescriptor,
    Buffer,
    BufferDescriptor,
    BufferUsages,
    IndexFormat,
    RenderPipeline,
    TextureUsages,
    TextureView,
    util::{
        BufferInitDescriptor,
        DeviceExt
    }
};

use crate::{
    frame::{
        FilterMode,
        WrapMode
    },
    wgpu_interface::WGPUInterface
};

pub struct Pipeline {
    pipeline: RenderPipeline,

    vertex_buffer: Buffer,
    index_buffer: Buffer,

    instance_buffer: Buffer,
    uniform_buffer: Buffer,

    uniform_bind_group: BindGroup,

    instance_buffer_counter: usize,
    uniform_buffer_counter: usize
}

pub const TEXTURE_BIND_GROUP_INDEX: u32 = 0;
pub const UNIFORM_BIND_GROUP_INDEX: u32 = 1;

impl Pipeline {
    pub fn create(wgpu_interface: &impl WGPUInterface,quad_instance_capacity: usize,uniform_capacity: usize) -> Self {

        let device = wgpu_interface.get_device();
        let pipeline = create_pipeline(wgpu_interface);

/*
  Triangle list should generate 0-1-2 2-1-3 in CCW

                    0---2
                    |  /|
                    | / |
                    |/  |
                    1---3
*/

        let vertices = vec![
            0.0,0.0, //Top Left     0
            0.0,1.0, //Bottom Left  1
            1.0,0.0, //Top Right    2
            1.0,1.0  //Bottom Right 3
        ];

        let indices = vec![
            0,1,2, //First Triangle
            2,1,3, //Second Triangle
            u16::MAX
        ];

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor{
            label: Some("Index Buffer"),
            contents: indices.as_bytes(),
            usage: wgpu::BufferUsages::INDEX
        });

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor{
            label: Some("Vertex Buffer"),
            contents: vertices.as_bytes(),
            usage: wgpu::BufferUsages::VERTEX
        });

        let instance_buffer = device.create_buffer(&BufferDescriptor{
            label: Some("Instance Buffer"),
            size: (size_of::<QuadInstance>() * quad_instance_capacity) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("View Projection Buffer"),
            size: (size_of::<ViewProjectionMatrix>() * uniform_capacity) as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &pipeline.get_bind_group_layout(UNIFORM_BIND_GROUP_INDEX),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("View Projection Bind Group"),
        });

        return Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            uniform_buffer,
            uniform_bind_group,
            instance_buffer_counter: usize::MIN,
            uniform_buffer_counter: usize::MIN
        };
    }

    pub fn get_texture_bind_group_layout(&self) -> BindGroupLayout {
        return self.pipeline.get_bind_group_layout(TEXTURE_BIND_GROUP_INDEX);
    }

    pub fn get_uniform_bind_group_layout(&self) -> BindGroupLayout {
        return self.pipeline.get_bind_group_layout(UNIFORM_BIND_GROUP_INDEX);
    }

    pub fn request_instance_buffer_start(&mut self,size: usize) -> usize {
        let index = self.instance_buffer_counter;
        self.instance_buffer_counter += size;
        return index;
    }

    pub fn request_uniform_buffer_start(&mut self,size: usize) -> usize {
        let index = self.uniform_buffer_counter;
        self.uniform_buffer_counter += size;
        return index;
    }

    pub fn reset_buffer_counters(&mut self) {
        self.instance_buffer_counter = 0;
        self.uniform_buffer_counter = 0;
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

    let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            &texture_bind_group_layout,
            &uniform_bind_group_layout,
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
            buffers: &[
                Vertex::get_buffer_layout(),
                QuadInstance::get_buffer_layout()
            ]
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
            topology: wgpu::PrimitiveTopology::TriangleList,
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


#[repr(C)]
#[derive(Copy,Clone,Debug,Default,Pod,Zeroable)]
pub struct Vertex {
    pub position: [f32;2]
}

#[repr(C)]
#[derive(Copy,Clone,Debug,Default,Pod,Zeroable)]
pub struct QuadInstance {
    pub position: [f32;2],
    pub size: [f32;2],
    pub uv_position: [f32;2],
    pub uv_size: [f32;2],
    pub rotation: f32,
    pub color: [f32;4],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute;1] = wgpu::vertex_attr_array![
        0 => Float32x2,
    ];

    pub fn get_buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        return wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

impl QuadInstance {
    const ATTRIBS: [wgpu::VertexAttribute;6] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x2,
        2 => Float32x2,
        3 => Float32x2,
        4 => Float32,
        5 => Float32x4
    ];

    pub fn get_buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        return wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

type ViewProjectionMatrix = [[f32;4];4];

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
