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
    ImageError,
    ImageReader
};

use wgpu::{
    BindGroup,
    BindGroupLayoutDescriptor,
    Buffer, BufferDescriptor,
    BufferUsages,
    CommandEncoder,
    CommandEncoderDescriptor,
    IndexFormat,
    RenderPass,
    RenderPipeline,
    util::{
        BufferInitDescriptor,
        DeviceExt
    }
};

use crate::{
    frame::{
        DrawData,
        Frame,
        FrameCreationOptions,
        FrameInternal
    },
    lease_arena::LeaseArena,
    texture_container::TextureContainer,
    wgpu_interface::WGPUInterface
};

const UNIFORM_BUFFER_ALIGNMENT: u32 = 256;

pub struct Pipeline {
    render_pipeline: RenderPipeline,

    vertex_buffer: Buffer,
    index_buffer: Buffer,

    instance_buffer: Buffer,
    uniform_buffer: Buffer,

    uniform_bind_group: BindGroup,

    instance_buffer_counter: u32,
    uniform_buffer_counter: u32,

    frame_cache: FrameCache,

    active: bool,
    encoder: Option<CommandEncoder>,
    output_frame_index: Option<Index>
}

type FrameCache = LeaseArena<(u32,u32),TextureContainer>;

pub struct PipelineCreationOptions {
    pub quad_instance_capacity: u32,
    pub uniform_capacity: u32,
    pub cache_options: Option<CacheOptions>
}

/* Reasonable-ish defaults. Callers, do it yourself! */
impl Default for PipelineCreationOptions {
    fn default() -> Self {
        Self {
            quad_instance_capacity: 640,
            uniform_capacity: 16,
            cache_options: None
        }
    }
}

pub struct CacheOptions {
    pub instances: u32,
    pub sizes: Vec<(u32,u32)>
}

#[allow(dead_code)]
impl Pipeline {

    pub const TEXTURE_BIND_GROUP_INDEX: u32 = 0;
    pub const UNIFORM_BIND_GROUP_INDEX: u32 = 1;

    pub fn create(
        wgpu_interface: &impl WGPUInterface,
        options: PipelineCreationOptions
    ) -> Self {
        return create_pipeline(wgpu_interface,options);
    }

    pub fn start(&mut self,wgpu_interface: &mut impl WGPUInterface) -> Option<Frame> {
        if self.active {
            panic!("Pipeline is already started. There is already a command encoder active.");
        }
        self.active = true;

        if self.output_frame_index.is_some() {
            panic!("Output frame already exists!");
        }

        if let Some(texture_container) = TextureContainer::create_output(wgpu_interface) {
            self.encoder = Some(wgpu_interface.get_device().create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder")
            }));
            let size = texture_container.size();
            let index = self.frame_cache.insert_keyless(texture_container);
            self.output_frame_index = Some(index);

            return Some(FrameInternal::create_output(size,index));
        } else {
            log::warn!("Unable to create output texture.");
            self.active = false;
            
            return None;
        }
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

    /* Persistent frames do not reuse the underlying mutable_textures pool. It is safe to use them across display frames. */
    pub fn get_persistent_frame(&mut self,wgpu_interface: &impl WGPUInterface,size: (u32,u32),write_once: bool) -> Frame {
        let frame = TextureContainer::create_mutable(
            wgpu_interface,
            &self.render_pipeline.get_bind_group_layout(Self::TEXTURE_BIND_GROUP_INDEX),
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
                &self.render_pipeline.get_bind_group_layout(Self::TEXTURE_BIND_GROUP_INDEX),
                size
            );
            let index = self.frame_cache.insert_leasable_and_take(size,new_texture);
            return FrameInternal::create(size,FrameCreationOptions { persistent: false, write_once, index });
        }
    }

    pub fn load_texture(&mut self,wgpu_interface: &impl WGPUInterface,path: &str) -> Result<Frame,ImageError> {
        let image = ImageReader::open(path)?.decode()?;
        let texture_container = TextureContainer::from_image(
            wgpu_interface,
            &self.render_pipeline.get_bind_group_layout(Self::TEXTURE_BIND_GROUP_INDEX),
            &image
        );
        return Ok(FrameInternal::create_texture(
            texture_container.size(),
            self.frame_cache.insert_keyless(texture_container)
        ));
    }

    fn request_instance_buffer_start(&mut self,quad_count: u32) -> u32 {
        let index = self.instance_buffer_counter;
        self.instance_buffer_counter += quad_count * size_of::<QuadInstance>() as u32;
        return index;
    }

    fn request_uniform_buffer_start(&mut self) -> u32 {
        let index = self.uniform_buffer_counter;
        self.uniform_buffer_counter += UNIFORM_BUFFER_ALIGNMENT;
        return index;
    }
}

pub trait PipelineInternal {
    fn get_texture_container(&self,reference: Index) -> &TextureContainer;
    fn create_render_pass<'a>(&mut self,wgpu_interface: &impl WGPUInterface,frame: &Frame,encoder: &'a mut CommandEncoder,) -> RenderPass<'a>;
    fn try_borrow_encoder(&mut self) -> Option<CommandEncoder>;
    fn return_encoder(&mut self,encoder: CommandEncoder);
    fn write_quad(&mut self,render_pass: &mut RenderPass,queue: &wgpu::Queue,draw_data: &DrawData);
    fn write_quad_set(&mut self,render_pass: &mut RenderPass,queue: &wgpu::Queue,draw_data: &[DrawData]);
}

impl PipelineInternal for Pipeline {

    fn get_texture_container(&self,reference: Index) -> &TextureContainer {
        return self.frame_cache.get(reference);
    }

    fn create_render_pass<'a>(
        &mut self,
        wgpu_interface: &impl WGPUInterface,
        frame: &Frame,
        encoder: &'a mut CommandEncoder,
    ) -> RenderPass<'a> {

        let texture_view = self.frame_cache.get(frame.get_index()).get_view();

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: match frame.get_clear_color() {
                        Some(color) => wgpu::LoadOp::Clear(color),
                        None => wgpu::LoadOp::Load,
                    },
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        let buffer_start: u32 = self.request_uniform_buffer_start();

        wgpu_interface.get_queue().write_buffer(
            &self.uniform_buffer,
            buffer_start as u64,
            bytemuck::bytes_of(&get_ortho_matrix(frame.size()))
        );

        //Index buffer
        render_pass.set_index_buffer(
            self.index_buffer.slice(..),
            wgpu::IndexFormat::Uint32
        );

        //Vertex buffer
        render_pass.set_vertex_buffer(0,self.vertex_buffer.slice(..));

        //Instance buffer
        render_pass.set_vertex_buffer(1,self.instance_buffer.slice(..));
        
        //Uniform buffer bind group
        render_pass.set_bind_group(
            Pipeline::UNIFORM_BIND_GROUP_INDEX,
            &self.uniform_bind_group,
            &[buffer_start]
        );

        return render_pass;
    }

    fn try_borrow_encoder(&mut self) -> Option<CommandEncoder> {
        if let Some(encoder) = self.encoder.take() {
            return Some(encoder);
        } else {
            return None;
        }
    }

    fn return_encoder(&mut self,encoder: CommandEncoder) {
        if !self.active {
            panic!("Pipeline was not started. There is no active command encoder.");
        }
        if self.encoder.is_some() {
            panic!("Pipeline already has a command encoder in place.");
        }
        self.encoder = Some(encoder);
    }

    fn write_quad(&mut self,render_pass: &mut RenderPass,queue: &wgpu::Queue,draw_data: &DrawData) {
        let quad_instance = &draw_data.to_quad_instance();
        let index = self.request_instance_buffer_start(1);
        queue.write_buffer(
            &self.instance_buffer,
            index as u64,
            bytemuck::bytes_of(quad_instance)
        );

        render_pass.draw_indexed(0..6,0,0..1);
    }

    fn write_quad_set(&mut self,render_pass: &mut RenderPass,queue: &wgpu::Queue,draw_data: &[DrawData]) {
        let mut quad_instances = Vec::with_capacity(draw_data.len());
        quad_instances.extend(draw_data.iter().map(|d|d.to_quad_instance()));

        let index = self.request_instance_buffer_start(draw_data.len() as u32);

        queue.write_buffer(
            &self.instance_buffer,
            index as u64,
            bytemuck::cast_slice(&quad_instances)
        );

        render_pass.draw_indexed(0..6,0,0..draw_data.len() as u32);
    }
}

fn get_ortho_matrix(size: (u32,u32)) -> [[f32;4];4] {
    let (width,height) = size;
    let matrix = cgmath::ortho(
        0.0, //Left
        width as f32, //Right
        height as f32, //Bottom
        0.0, //Top
        -1.0, //Near
        1.0, //Far
    ).into();
    return matrix;
}

fn create_pipeline(
    wgpu_interface: &impl WGPUInterface,
    options: PipelineCreationOptions
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

    let vertices = [
        -0.5,-0.5, //Top Left     0
        -0.5, 0.5, //Bottom Left  1
         0.5,-0.5, //Top Right    2
         0.5, 0.5  //Bottom Right 3
    ];

    let indices: [u32;6] = [
        0,1,2,
        2,1,3
    ];

    let index_buffer = device.create_buffer_init(&BufferInitDescriptor{
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX
    });

    let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor{
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX
    });

    let instance_buffer = device.create_buffer(&BufferDescriptor{
        label: Some("Instance Buffer"),
        size: (size_of::<QuadInstance>() * options.quad_instance_capacity as usize) as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let uniform_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("View Projection Buffer"),
        //See: https://docs.rs/wgpu-types/27.0.1/wgpu_types/struct.Limits.html#structfield.min_storage_buffer_offset_alignment
        size: (UNIFORM_BUFFER_ALIGNMENT * options.uniform_capacity) as u64,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false
    });

    let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &render_pipeline.get_bind_group_layout(Pipeline::UNIFORM_BIND_GROUP_INDEX),
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
        label: Some("View Projection Bind Group"),
    });

    let frame_cache: FrameCache = match options.cache_options {
        None => FrameCache::default(),
        Some(cache_options) => {
            let capacity = cache_options.sizes.len();
            let instances = cache_options.instances;

            if instances < 1 {
                log::warn!("Frame cache instances is 0. No caches will be created.");
            }

            let mut textures = Arena::with_capacity(capacity);
            let mut mutable_textures = HashMap::with_capacity(capacity);

            let bind_group_layout = &render_pipeline.get_bind_group_layout(Pipeline::TEXTURE_BIND_GROUP_INDEX);

            for size in cache_options.sizes.iter() {
                let mut queue = VecDeque::with_capacity(instances as usize);

                for _ in 0..instances {

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
        instance_buffer_counter: 0,
        uniform_buffer_counter: 0
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
                has_dynamic_offset: true,
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
            strip_index_format: Some(IndexFormat::Uint32),
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
pub struct Vertex { // Aligned to 16
    pub position: [f32;2],
    _padding: [f32;2]
}

#[repr(C)]
#[derive(Copy,Clone,Debug,Default,Pod,Zeroable)]
pub struct QuadInstance { //Aligned to 64
    pub position: [f32;2],
    pub size: [f32;2],
    pub uv_position: [f32;2],
    pub uv_size: [f32;2],
    pub color: [f32;4],
    pub rotation: f32,
    pub _padding: [f32;3]
}

#[non_exhaustive]
struct ATTR;

impl ATTR {
    pub const VERTEX_POSITION: u32 = 0;
    pub const INSTANCE_POSITION: u32 = 1;
    pub const SIZE: u32 = 2;
    pub const UV_POS: u32 = 3;
    pub const UV_SIZE: u32 = 4;
    pub const COLOR: u32 = 5;
    pub const ROTATION: u32 = 6;
}

impl Vertex {
    const ATTRS: [wgpu::VertexAttribute;1] = wgpu::vertex_attr_array![
        ATTR::VERTEX_POSITION => Float32x2,
    ];

    pub fn get_buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        return wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRS,
        }
    }
}

impl QuadInstance {
    const ATTRS: [wgpu::VertexAttribute;6] = wgpu::vertex_attr_array![
        ATTR::INSTANCE_POSITION => Float32x2,
        ATTR::SIZE => Float32x2,
        ATTR::UV_POS => Float32x2,
        ATTR::UV_SIZE => Float32x2,
        ATTR::COLOR => Float32x4,
        ATTR::ROTATION => Float32,
    ];

    pub fn get_buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        return wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRS,
        }
    }
}
