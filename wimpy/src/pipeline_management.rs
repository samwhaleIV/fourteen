use core::panic;
use std::collections::{
    HashMap,
    VecDeque
};

use bytemuck::{
    Pod,
    Zeroable
};

use generational_arena::{Arena, Index};

use image::{
    EncodableLayout,
    ImageError,
    ImageReader
};

use wgpu::{
    BindGroup, BindGroupLayout, BindGroupLayoutDescriptor, Buffer, BufferDescriptor, BufferUsages, CommandEncoder, CommandEncoderDescriptor, IndexFormat, RenderPipeline, util::{
        BufferInitDescriptor,
        DeviceExt
    }
};

use crate::{
    frame::{
        Frame, FrameCreationOptions, FrameInternal
    },
    lease_arena::LeaseArena,
    texture_container::TextureContainer,
    wgpu_interface::WGPUInterface
};

pub struct Pipeline {
    render_pipeline: RenderPipeline,

    vertex_buffer: Buffer,
    index_buffer: Buffer,

    instance_buffer: Buffer,
    uniform_buffer: Buffer,

    uniform_bind_group: BindGroup,

    instance_buffer_counter: usize,
    uniform_buffer_counter: usize,

    frame_cache: FrameCache,

    active: bool,
    encoder: Option<CommandEncoder>,
    output_frame_index: Option<Index>
}

type FrameCache = LeaseArena<(u32,u32),TextureContainer>;

pub const TEXTURE_BIND_GROUP_INDEX: u32 = 0;
pub const UNIFORM_BIND_GROUP_INDEX: u32 = 1;

struct FrameCacheConfig<'a> {
    sizes: &'a[(u32,u32)],
    instances: usize
}

impl Pipeline {
    pub fn create(
        wgpu_interface: &impl WGPUInterface,
        quad_instance_capacity: usize,
        uniform_capacity: usize
    ) -> Self {
        return create_pipeline(wgpu_interface,quad_instance_capacity,uniform_capacity,None);
    }

    pub fn create_with_buffer_frames(
        wgpu_interface: &impl WGPUInterface,
        quad_instance_capacity: usize,
        uniform_capacity: usize,
        cache_sizes: &[(u32,u32)],
        cache_instances: usize
    ) -> Self {
        return create_pipeline(wgpu_interface,quad_instance_capacity,uniform_capacity,Some(FrameCacheConfig {
            sizes: cache_sizes,
            instances: cache_instances
        }));
    }

    pub fn start(&mut self,wgpu_interface: &mut impl WGPUInterface) -> Frame {
        if self.active {
            panic!("Pipeline is already started. There is already a command encoder active.");
        }
     
        self.encoder = Some(wgpu_interface.get_device().create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder")
        }));
        self.active = true;

        return self.get_output_frame(wgpu_interface);
    }

    pub fn finish(&mut self,wgpu_interface: &mut impl WGPUInterface) {
        if !self.active {
            panic!("Pipeline was not started. There is no active command encoder.");
        }
        if let Some(encoder) = self.encoder.take() {
            wgpu_interface.get_queue().submit(std::iter::once(encoder.finish()));

            self.frame_cache.end_all_leases();

            self.instance_buffer_counter = 0;
            self.uniform_buffer_counter = 0;

            if let Some(index) = self.output_frame_index.take() {
                self.frame_cache.remove(index);
            } else {
                log::warn!("Output frame index not found on frame cleanup.");
            }

            self.encoder = None;
            self.active = false;
        } else {
            panic!("Encoder not found. Did a caller forget to return it?");
        }
    }

    pub fn try_borrow_encoder(&mut self) -> Option<CommandEncoder> {
        if let Some(encoder) = self.encoder.take() {
            return Some(encoder);
        } else {
            return None;
        }
    }

    pub fn return_encoder(&mut self,encoder: CommandEncoder) {
        if !self.active {
            panic!("Pipeline was not started. There is no active command encoder.");
        }
        if self.encoder.is_some() {
            panic!("Pipeline already has a command encoder in place.");
        }
        self.encoder = Some(encoder);
    }

    pub fn get_texture_bind_group_layout(&self) -> BindGroupLayout {
        return self.render_pipeline.get_bind_group_layout(TEXTURE_BIND_GROUP_INDEX);
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

    fn get_output_frame(&mut self,wgpu_interface: &impl WGPUInterface) -> Frame {
        if self.output_frame_index.is_some() {
            panic!("Output frame already exists!");
        }
        let texture_container = TextureContainer::create_output(wgpu_interface);
        let size = texture_container.size();

        let index = self.frame_cache.insert_keyless(texture_container);

        self.output_frame_index = Some(index);

        return FrameInternal::create_output(size,index);
    }

    /* Persistent frames do not reuse the underlying mutable_textures pool. It is safe to use them across display frames. */
    pub fn get_persistent_frame(&mut self,wgpu_interface: &impl WGPUInterface,size: (u32,u32),write_once: bool) -> Frame {
        let frame = TextureContainer::create_mutable(
            wgpu_interface,
            &self.render_pipeline.get_bind_group_layout(TEXTURE_BIND_GROUP_INDEX),
            size
        );
        let index = self.frame_cache.insert_keyless(frame);
        return FrameInternal::create(size,FrameCreationOptions {
            persistent: true,
            index,
            write_once
        });
    }

    pub fn get_temp_frame(&mut self,wgpu_interface: &impl WGPUInterface,size: (u32,u32),write_once: bool) -> Frame {
        if let Some(index) = self.frame_cache.try_request_lease(size) {
            return FrameInternal::create(size,FrameCreationOptions { persistent: false, write_once, index });
        } else {
            let new_texture = TextureContainer::create_mutable(
                wgpu_interface,
                &self.render_pipeline.get_bind_group_layout(TEXTURE_BIND_GROUP_INDEX),
                size
            );
            let index = self.frame_cache.insert_leasable_and_take(size,new_texture);
            return FrameInternal::create(size,FrameCreationOptions { persistent: false, write_once, index });
        }
    }

    pub fn get_texture_container(&self,reference: Index) -> &TextureContainer {
        return self.frame_cache.get(reference);
    }

    pub fn load_texture(&mut self,wgpu_interface: &impl WGPUInterface,path: &str) -> Result<Frame,ImageError> {
        let image = ImageReader::open(path)?.decode()?;
        let texture_container = TextureContainer::from_image(
            wgpu_interface,
            &self.render_pipeline.get_bind_group_layout(TEXTURE_BIND_GROUP_INDEX),
            &image
        );
        return Ok(FrameInternal::create_texture(
            texture_container.size(),
            self.frame_cache.insert_keyless(texture_container)
        ));
    }
}

fn create_pipeline(
    wgpu_interface: &impl WGPUInterface,
    quad_instance_capacity: usize,
    uniform_capacity: usize,
    cache_config: Option<FrameCacheConfig>
) -> Pipeline {

    let device = wgpu_interface.get_device();
    let render_pipeline = create_wgpu_pipeline(wgpu_interface);

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
        layout: &render_pipeline.get_bind_group_layout(UNIFORM_BIND_GROUP_INDEX),
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
        label: Some("View Projection Bind Group"),
    });

    let frame_cache: FrameCache = match cache_config {
        None => FrameCache::default(),
        Some(config) => {
            let capacity = config.sizes.len();

            let mut textures = Arena::with_capacity(capacity);
            let mut mutable_textures = HashMap::with_capacity(capacity);

            let bind_group_layout = &render_pipeline.get_bind_group_layout(TEXTURE_BIND_GROUP_INDEX);

            for size in config.sizes.iter() {
                let mut queue = VecDeque::with_capacity(config.instances);

                for _ in 0..config.instances {

                    let mutable_texture = TextureContainer::create_mutable(
                        wgpu_interface,
                        bind_group_layout,
                        *size
                    );

                    let index = textures.insert(mutable_texture);
                    queue.push_back(index);
                }

                mutable_textures.insert(*size,queue);
            }

            FrameCache::create_with_values(textures,mutable_textures)
        }
    };

    return Pipeline {
        render_pipeline,
        vertex_buffer,
        index_buffer,
        instance_buffer,
        uniform_buffer,
        uniform_bind_group,
        frame_cache,
        encoder: None,
        active: false,
        output_frame_index: None,
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
