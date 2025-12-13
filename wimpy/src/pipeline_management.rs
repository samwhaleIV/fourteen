use std::collections::{
    HashMap,
    VecDeque
};

use bytemuck::{
    Pod,
    Zeroable
};

use generational_arena::{
    Arena,
    Index
};

use image::{
    DynamicImage,
    EncodableLayout,
    ImageError,
    ImageReader
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
    util::{
        BufferInitDescriptor,
        DeviceExt
    }
};

use crate::{
    frame::{
        Frame,
        FrameInternal
    },
    lease_arena::LeaseArena,
    texture_container::TextureContainer,
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
    uniform_buffer_counter: usize,

    frame_cache: FrameCache
}

type FrameCache = LeaseArena<(u32,u32),TextureContainer>;

pub const TEXTURE_BIND_GROUP_INDEX: u32 = 0;
pub const UNIFORM_BIND_GROUP_INDEX: u32 = 1;

impl Pipeline {

    pub fn create_with_buffer_frames(
        wgpu_interface: &impl WGPUInterface,
        quad_instance_capacity: usize,
        uniform_capacity: usize,
        cache_sizes: &[(u32,u32)],
        cache_size_instances: usize
    ) -> Self {
        return create_pipeline(
            wgpu_interface,
            quad_instance_capacity,
            uniform_capacity,
            create_frame_cache_init(
                wgpu_interface,
                cache_sizes,
                cache_size_instances
            )
        );
    }

    pub fn create(
        wgpu_interface: &impl WGPUInterface,
        quad_instance_capacity: usize,
        uniform_capacity: usize
    ) -> Self {
        return create_pipeline(
            wgpu_interface,
            quad_instance_capacity,
            uniform_capacity,
            FrameCache::default()
        );
    }

    pub fn start(&self,wgpu_interface: &mut impl WGPUInterface) -> Frame {
        wgpu_interface.start_encoding();
        return self.get_output_frame(wgpu_interface);
    }

    pub fn finish(&mut self,wgpu_interface: &mut impl WGPUInterface) {
        self.instance_buffer_counter = 0;
        self.uniform_buffer_counter = 0;
        wgpu_interface.finish_encoding();
    }
}

pub trait PipelineResourceManagement {
    fn get_texture_bind_group_layout(&self) -> BindGroupLayout;
    fn get_uniform_bind_group_layout(&self) -> BindGroupLayout;
    fn request_instance_buffer_start(&mut self,size: usize) -> usize;
    fn request_uniform_buffer_start(&mut self,size: usize) -> usize;
}

impl PipelineResourceManagement for Pipeline {
    fn get_texture_bind_group_layout(&self) -> BindGroupLayout {
        return self.pipeline.get_bind_group_layout(TEXTURE_BIND_GROUP_INDEX);
    }

    fn get_uniform_bind_group_layout(&self) -> BindGroupLayout {
        return self.pipeline.get_bind_group_layout(UNIFORM_BIND_GROUP_INDEX);
    }

    fn request_instance_buffer_start(&mut self,size: usize) -> usize {
        let index = self.instance_buffer_counter;
        self.instance_buffer_counter += size;
        return index;
    }

    fn request_uniform_buffer_start(&mut self,size: usize) -> usize {
        let index = self.uniform_buffer_counter;
        self.uniform_buffer_counter += size;
        return index;
    }
}

pub fn create_frame_cache_init(wgpu_interface: &impl WGPUInterface,cache_sizes: &[(u32,u32)],cache_size_instances: usize) -> FrameCache {

    let capacity = cache_sizes.len();

    let mut textures = Arena::with_capacity(capacity);
    let mut mutable_textures = HashMap::with_capacity(capacity);

    for size in cache_sizes.iter() {
        let mut queue = VecDeque::with_capacity(cache_size_instances);

        for _ in 0..cache_size_instances {
            let mutable_texture = TextureContainer::create_mutable(*size,wgpu_interface);
            let index = textures.insert(mutable_texture);
            queue.push_back(index);
        }

        mutable_textures.insert(*size,queue);
    }

    let frames = LeaseArena::create_with_values(textures,mutable_textures);

    return frames;
}

pub trait FrameCacheManagement {
    fn get_output_frame(&self,wgpu_interface: &impl WGPUInterface) -> Frame;
    fn create_frame_static(&self,size: (u32,u32),readonly_after_render: bool) -> Frame;
    fn get_mutable_texture_lease(&mut self,wgpu_interface: &impl WGPUInterface,size: (u32,u32)) -> Index;
    fn return_mutable_texture_lease(&mut self,lease: Index);
    fn get_texture(&self,reference: Index) -> &TextureContainer;
    fn create_finished_frame(&mut self,image: &DynamicImage,wgpu_interface: &impl WGPUInterface) -> Frame;
    fn create_texture_frame(&mut self,name: &str,wgpu_interface: &impl WGPUInterface) -> Result<Frame,ImageError>;
    fn create_texture_frame_debug(&mut self,wgpu_interface: &impl WGPUInterface) -> Frame;
}

impl FrameCacheManagement for Pipeline {
    fn get_output_frame(&self,wgpu_interface: &impl WGPUInterface) -> Frame {
        return FrameInternal::create_output(wgpu_interface);
    }

    /* Non-statics do not reuse the underlying mutable_textures pool. It is safe to use them across display frames. */
    fn create_frame_static(&self,size: (u32,u32),readonly_after_render: bool) -> Frame {
        return match readonly_after_render {
            true => FrameInternal::create_immutable(size,true),
            false => FrameInternal::create_mutable(size),
        }
    }

    fn get_mutable_texture_lease(&mut self,wgpu_interface: &impl WGPUInterface,size: (u32,u32)) -> Index {
        return self.frame_cache.start_lease(size,||TextureContainer::create_mutable(size,wgpu_interface));
    }
  
    fn return_mutable_texture_lease(&mut self,lease: Index) {
        self.frame_cache.end_lease(lease);
    }

    fn get_texture(&self,reference: Index) -> &TextureContainer {
        return self.frame_cache.get(reference);
    }

    fn create_finished_frame(&mut self,image: &DynamicImage,wgpu_interface: &impl WGPUInterface) -> Frame {
        let texture_container = TextureContainer::from_image(&image,wgpu_interface);
        let size = texture_container.size();
        let index = self.frame_cache.insert(size,texture_container);
        return Frame::to_immutable(size,index);
    }

    fn create_texture_frame(&mut self,name: &str,wgpu_interface: &impl WGPUInterface) -> Result<Frame,ImageError> {
        let image = ImageReader::open(name)?.decode()?;
        let frame = self.create_finished_frame(&image,wgpu_interface);
        return Ok(frame);
    }

    fn create_texture_frame_debug(&mut self,wgpu_interface: &impl WGPUInterface) -> Frame {
        let bytes = include_bytes!("../../content/images/null.png");
        let image = image::load_from_memory(bytes).unwrap();
        let frame = self.create_finished_frame(&image,wgpu_interface);
        return frame;
    }
}

fn create_pipeline(
    wgpu_interface: &impl WGPUInterface,
    quad_instance_capacity: usize,
    uniform_capacity: usize,
    frame_cache: FrameCache
) -> Pipeline {

    let device = wgpu_interface.get_device();
    let pipeline = create_wgpu_pipeline(wgpu_interface);

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

    return Pipeline {
        pipeline,
        vertex_buffer,
        index_buffer,
        instance_buffer,
        uniform_buffer,
        uniform_bind_group,
        frame_cache,
        instance_buffer_counter: usize::MIN,
        uniform_buffer_counter: usize::MIN
    };
}

pub fn create_wgpu_pipeline(wgpu_interface: &impl WGPUInterface) -> RenderPipeline {

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
