use core::panic;

use bytemuck::{
    Pod,
    Zeroable
};

use wgpu::{
    BindGroup,
    BindGroupLayoutDescriptor,
    Buffer, BufferDescriptor,
    BufferUsages,
    CommandEncoder,
    CommandEncoderDescriptor,
    RenderPass,
    RenderPipeline,
    SurfaceTexture,
    util::{
        BufferInitDescriptor,
        DeviceExt
    }
};

use crate::{
    internal::CachesArena,
    wgpu::{
        FilterMode,
        WrapMode,
        frame::FrameCommand,
        texture_container::SamplerMode
    }
};

use super::{
    texture_container::TextureContainer,
    wgpu_handle::WGPUHandle,
    frame::{
        Frame,
        FrameInternal,
        FrameCreationOptions,
        DrawData
    }
};

const UNIFORM_BUFFER_ALIGNMENT: u32 = 256;

slotmap::new_key_type! {
    pub struct FrameCacheReference;
}

pub struct GraphicsContext<THandle> {
    render_pipeline: RenderPipeline,

    vertex_buffer: Buffer,
    index_buffer: Buffer,

    instance_buffer: Buffer,
    uniform_buffer: Buffer,

    wgpu_handle: Option<THandle>,

    uniform_bind_group: BindGroup,

    instance_buffer_write_offset: u32,
    uniform_buffer_write_offset: u32,

    frame_cache: FrameCache,

    active: bool,
    encoder: Option<CommandEncoder>,

    output_frame: Option<OutputFrame>
}

struct OutputFrame {
    cache_reference: FrameCacheReference,
    surface: SurfaceTexture
}

type FrameCache = CachesArena<(u32,u32),FrameCacheReference,TextureContainer>;

pub struct GraphicsContextConfig<'a> {
    pub quad_instance_capacity: u32,
    pub uniform_capacity: u32,
    pub cache: Option<CacheConfig<'a>>
}

/* Reasonable-ish defaults. Callers, do it yourself! */
impl Default for GraphicsContextConfig<'_> {
    fn default() -> Self {
        Self {
            quad_instance_capacity: 640,
            uniform_capacity: 16,
            cache: None
        }
    }
}

pub struct CacheConfig<'a> {
    pub instances: u32,
    pub sizes: &'a[(u32,u32)]
}

pub enum FrameLifetime {
    Temporary,
    Persistent
}

pub struct FrameConfig {
    pub lifetime: FrameLifetime,
    pub size: (u32,u32),
    pub draw_once: bool
}

pub const TEXTURE_BIND_GROUP_INDEX: u32 = 0;
pub const UNIFORM_BIND_GROUP_INDEX: u32 = 1;

impl<THandle: WGPUHandle> GraphicsContext<THandle> {

    pub fn create(
        wgpu_handle: &THandle,
        options: GraphicsContextConfig
    ) -> Self {
        return create_graphics_context(wgpu_handle,options);
    }

    pub fn create_output_frame(&mut self) -> Option<Frame> {
        if self.active {
            panic!("Pipeline is already started. There is already a command encoder active.");
        }
        self.active = true;

        if self.output_frame.is_some() {
            panic!("Output frame already exists!");
        }

        if let Some(wgpu_handle) = &self.wgpu_handle && let Some(surface) = wgpu_handle.get_output_surface() {
            self.encoder = Some(wgpu_handle.get_device().create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder")
            }));
            let size = (
                surface.texture.width(),
                surface.texture.height()
            );
            let texture_container = TextureContainer::create_output(&surface,size);
            let index = self.frame_cache.insert_keyless(texture_container);

            self.output_frame = Some(OutputFrame { cache_reference: index, surface });
            return Some(FrameInternal::create_output(size,index));
        } else {
            log::warn!("Unable to create output texture.");
            self.active = false;
            
            return None;
        }
    }

    pub fn bake(&mut self,frame: &mut Frame) {
        frame.finish(self);
    }

    pub fn present_output_frame(&mut self) {
        if !self.active {
            panic!("Pipeline was not started. There is no active command encoder.");
        }
        if let (Some(wgpu_handle),Some(encoder)) = (self.wgpu_handle.take(),self.encoder.take()) {
            wgpu_handle.get_queue().submit(std::iter::once(encoder.finish()));

            self.frame_cache.end_all_leases();

            self.instance_buffer_write_offset = 0;
            self.uniform_buffer_write_offset = 0;

            if let Some(output_frame) = self.output_frame.take() {
                self.frame_cache.remove(output_frame.cache_reference);
                output_frame.surface.present();
            } else {
                log::warn!("Output frame not found during frame finish.");
            }
            self.active = false;
        } else {
            panic!("Encoder or wgpu handle not found. Did a caller forget to return it?");
        }
    }

    /* Persistent frames do not reuse the underlying mutable_textures pool. It is safe to use them across display frames. */
    fn get_persistent_frame(&mut self,wgpu_handle: &THandle,size: (u32,u32),write_once: bool) -> Frame {
        let frame = TextureContainer::create_mutable(
            wgpu_handle,
            &self.render_pipeline.get_bind_group_layout(TEXTURE_BIND_GROUP_INDEX),
            size
        );
        let index = self.frame_cache.insert_keyless(frame);
        return FrameInternal::create(size,FrameCreationOptions {
            persistent: true,
            cache_reference: index,
            write_once
        });
    }

    fn get_temp_frame(&mut self,wgpu_handle: &THandle,size: (u32,u32),write_once: bool) -> Frame {
        return match self.frame_cache.start_lease(size) {
            Some(cache_reference) => {
                FrameInternal::create(size,FrameCreationOptions { persistent: false, write_once, cache_reference })
            },
            None => {
                let new_texture = TextureContainer::create_mutable(
                    wgpu_handle,
                    &self.render_pipeline.get_bind_group_layout(TEXTURE_BIND_GROUP_INDEX),
                    size
                );
                let index = self.frame_cache.insert_with_lease(size,new_texture);
                FrameInternal::create(size,FrameCreationOptions { persistent: false, write_once, cache_reference: index })
            },
        };
    }

    pub fn get_frame(&mut self,config: FrameConfig) -> Option<Frame> {
        if let Some(wgpu_handle) = self.wgpu_handle.take() {
            let frame = Some(match config.lifetime {
                FrameLifetime::Temporary => {
                    self.get_temp_frame(&wgpu_handle,config.size,config.draw_once)
                },
                FrameLifetime::Persistent => {
                    self.get_persistent_frame(&wgpu_handle,config.size,config.draw_once)
                },
            });
            self.wgpu_handle = Some(wgpu_handle);
            return frame;
        } else {
            log::error!("Failure to create frame. Missing WGPU handle.");
            return None;
        }
    }

    pub fn ensure_cached_frames(&mut self,sizes: &[(u32,u32)],instances: u32) {

    }

    pub fn load_texture(&mut self,bytes: &[u8]) -> Option<Frame> {
        if let Some(wgpu_handle) = self.wgpu_handle.take() {
            let image = image::load_from_memory(bytes).unwrap();
            let texture_container = TextureContainer::from_image(
                &wgpu_handle,
                &self.render_pipeline.get_bind_group_layout(TEXTURE_BIND_GROUP_INDEX),
                &image
            );
            self.wgpu_handle = Some(wgpu_handle);
            return Some(FrameInternal::create_texture(
                texture_container.size(),
                self.frame_cache.insert_keyless(texture_container)
            ));
        } else {
            return None;
        }
    }

    fn request_instance_buffer_start(&mut self,quad_count: u32) -> u32 {
        let index = self.instance_buffer_write_offset;
        self.instance_buffer_write_offset += quad_count;
        return index;
    }

    fn request_uniform_buffer_start(&mut self) -> u32 {
        let index = self.uniform_buffer_write_offset;
        self.uniform_buffer_write_offset += 1;
        return index;
    }

    fn get_texture_container(&self,cache_reference: FrameCacheReference) -> &TextureContainer {
        return self.frame_cache.get(cache_reference);
    }

    fn create_render_pass<'a>(
        &mut self,
        frame: &Frame,
        queue: &wgpu::Queue,
        encoder: &'a mut CommandEncoder,
    ) -> RenderPass<'a> {

        let texture_view = self.frame_cache.get(frame.get_cache_reference()).get_view();

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
        render_pass.set_pipeline(&self.render_pipeline);

        let buffer_start: u32 = self.request_uniform_buffer_start();

        queue.write_buffer(
            &self.uniform_buffer,
            (buffer_start * UNIFORM_BUFFER_ALIGNMENT) as u64,
            bytemuck::bytes_of(&get_camera_uniform(frame.size()))
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
            UNIFORM_BIND_GROUP_INDEX,
            &self.uniform_bind_group,
            &[buffer_start]
        );

        return render_pass;
    }

    fn write_quad(&mut self,render_pass: &mut RenderPass,queue: &wgpu::Queue,draw_data: &DrawData) {
        let quad_instance = draw_data.to_quad_instance();
        let index = self.request_instance_buffer_start(1);
        queue.write_buffer(
            &self.instance_buffer,
            index as u64 * size_of::<QuadInstance>() as u64,
            bytemuck::bytes_of(&quad_instance)
        );
        render_pass.draw_indexed(0..6,0,index..index + 1);
    }

    fn write_quad_set(&mut self,render_pass: &mut RenderPass,queue: &wgpu::Queue,draw_data: &[DrawData]) {
        let mut quad_instances = Vec::with_capacity(draw_data.len());
        quad_instances.extend(draw_data.iter().map(|d|d.to_quad_instance()));

        let index = self.request_instance_buffer_start(draw_data.len() as u32);

        queue.write_buffer(
            &self.instance_buffer,
            index as u64 * size_of::<QuadInstance>() as u64,
            bytemuck::cast_slice(&quad_instances)
        );

        render_pass.draw_indexed(0..6,0,index..index + draw_data.len() as u32);
    }

    fn process_commands(&mut self,render_pass: &mut RenderPass,frame: &Frame,queue: &wgpu::Queue) {
        let mut needs_sampler_update: bool = true;

        let mut filter_mode: FilterMode = FilterMode::Nearest;
        let mut wrap_mode: WrapMode = WrapMode::Clamp;

        let mut current_sampling_frame: Option<FrameCacheReference> = None;

        /* Some deeply complex optimization option could coalesce commands together, but set commands should cover any optimization concerns. */

        for command in frame.get_command_buffer().iter() {

            if let Some(new_index) = match command {
                FrameCommand::DrawFrame(index,_) |
                FrameCommand::DrawFrameSet(index,_) => Some(index),
                //Add more types if they change the sampler bind group
                _ => None
            } {
                let texture_container = self.get_texture_container(*new_index);

                if needs_sampler_update || match current_sampling_frame.take() {
                    Some(current_index) => current_index != *new_index,
                    None => true
                } {
                    let sampler_mode = SamplerMode::get_mode(filter_mode,wrap_mode);
                    let sampler = texture_container.get_bind_group(sampler_mode);
                    render_pass.set_bind_group(TEXTURE_BIND_GROUP_INDEX,sampler,&[]);
                }
                needs_sampler_update = false;
                current_sampling_frame = Some(*new_index);
            }

            match command {
                FrameCommand::SetTextureFilter(value) => {
                    if filter_mode != *value {
                        filter_mode = *value;
                        needs_sampler_update = true;
                    }
                },

                FrameCommand::SetTextureWrap(value) => {
                    if wrap_mode != *value {
                        wrap_mode = *value;
                        needs_sampler_update = true;
                    }
                },

                FrameCommand::DrawFrame(_,draw_data) => {
                    self.write_quad(render_pass,queue,draw_data);
                },

                FrameCommand::DrawFrameSet(_,draw_data_set) => {
                    self.write_quad_set(render_pass,queue,&draw_data_set);
                },
            }
        }
    }
}

pub trait GraphicsContextInternal<THandle> {
    fn insert_wgpu_handle(&mut self,wgpu_handle: THandle);
    fn remove_wgpu_handle(&mut self) -> Option<THandle>;
    fn render_frame(&mut self,frame: &Frame);
}

impl<THandle> GraphicsContextInternal<THandle> for GraphicsContext<THandle> where THandle: WGPUHandle {
    fn insert_wgpu_handle(&mut self,wgpu_handle: THandle) {
        self.wgpu_handle = Some(wgpu_handle);
    }
    fn remove_wgpu_handle(&mut self) -> Option<THandle> {
        return self.wgpu_handle.take();
    }
    fn render_frame(&mut self,frame: &Frame) {
        /* This is not where the encoder is created. Only 1 encoder is created for the master, output frame. */
        if let (Some(wgpu_handle),Some(mut encoder)) = (self.wgpu_handle.take(),self.encoder.take()) {
            {
                let queue = wgpu_handle.get_queue();
                let mut render_pass = self.create_render_pass(frame,queue,&mut encoder);
                self.process_commands(&mut render_pass,frame,queue);
            }
            self.wgpu_handle = Some(wgpu_handle);
            self.encoder = Some(encoder);
        } else {
            panic!("Missing wgpu handle or encoder.");
        }
    }
}

fn get_camera_uniform(size: (u32,u32)) -> CameraUniform {
    let (width,height) = size;

    let ortho = cgmath::ortho(
        0.0, //Left
        width as f32, //Right
        height as f32, //Bottom
        0.0, //Top
        -1.0, //Near
        1.0, //Far
    );

    return CameraUniform { view_projection: ortho.into() };
}

fn create_graphics_context<THandle: WGPUHandle> (
    wgpu_handle: &THandle,
    config: GraphicsContextConfig
) -> GraphicsContext<THandle> {

    let device = wgpu_handle.get_device();
    let render_pipeline = create_wgpu_pipeline(wgpu_handle);

/*
Triangle list should generate 0-1-2 2-1-3 in CCW

                0---2
                |  /|
                | / |
                |/  |
                1---3
*/

    let vertices = [  
        Vertex { position: [-0.5,-0.5] }, //Top Left     0
        Vertex { position: [-0.5, 0.5] }, //Bottom Left  1
        Vertex { position: [0.5,-0.5] }, //Top Right    2
        Vertex { position: [0.5, 0.5] }  //Bottom Right 3
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
        size: (size_of::<QuadInstance>() * config.quad_instance_capacity as usize) as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let uniform_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("View Projection Buffer"),
        //See: https://docs.rs/wgpu-types/27.0.1/wgpu_types/struct.Limits.html#structfield.min_storage_buffer_offset_alignment
        size: (UNIFORM_BUFFER_ALIGNMENT * config.uniform_capacity) as u64,
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

    
    let mut frame_cache = FrameCache::default();

    if let Some(cache_config) = config.cache {
        let instances = cache_config.instances;
        if instances >= 1 {
            log::warn!("Frame cache instances is 0. No caches will be created.");
        } else {
            let bind_group_layout = &render_pipeline.get_bind_group_layout(TEXTURE_BIND_GROUP_INDEX);

            for size in cache_config.sizes.iter() {
                for _ in 0..instances {

                    let mutable_texture = TextureContainer::create_mutable(
                        wgpu_handle,
                        bind_group_layout,
                        *size
                    );
                    frame_cache.insert(*size,mutable_texture);
                }
            }
        }
    };

    return GraphicsContext {
        wgpu_handle: None,
        render_pipeline,
        vertex_buffer,
        index_buffer,
        instance_buffer,
        uniform_buffer,
        uniform_bind_group,
        frame_cache,
        encoder: None,
        active: false,
        output_frame: None,
        instance_buffer_write_offset: 0,
        uniform_buffer_write_offset: 0
    };
}

pub fn create_wgpu_pipeline<THandle: WGPUHandle>(wgpu_handle: &THandle) -> RenderPipeline {

    let device = wgpu_handle.get_device();

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
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/quads.wgsl").into())
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
                format: wgpu_handle.get_output_format(),
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })]
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,

             /* Only for PrimitiveTopology::TriangleStrip */
            strip_index_format: None,

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
    //_padding: [f32;2]
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

#[repr(C)]
#[derive(Debug,Copy,Clone,Pod,Zeroable)]
pub struct CameraUniform {
    view_projection: [[f32;4];4]
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
