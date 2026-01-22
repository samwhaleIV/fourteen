use std::ops::Range;

use wgpu::{
    BindGroup, BindGroupLayoutDescriptor, Buffer, BufferDescriptor, BufferUsages, CommandEncoder, CommandEncoderDescriptor, RenderPass, RenderPipeline, SurfaceError, SurfaceTexture, util::{
        BufferInitDescriptor,
        DeviceExt
    }
};

use crate::{shared::CacheArenaError, wgpu::{
    DrawData, FrameError, command_processor::process_frame_commands, constants::{
        BindGroupIndices,
        INDEX_BUFFER_SIZE,
        UNIFORM_BUFFER_ALIGNMENT
    }, double_buffer::DoubleBuffer, frame_cache::{
        FrameCache,
        FrameCacheReference
    }, shader_definitions::{
        CameraUniform,
        QuadInstance,
        Vertex
    }
}};

use super::{
    texture_container::TextureContainer,
    graphics_provider::GraphicsProvider,
    frame::{
        Frame,
        FrameInternal,
        FrameCreationOptions,
    }
};

pub struct DoubleBufferSet {
    instances: DoubleBuffer<QuadInstance>,
    uniforms: DoubleBuffer<CameraUniform>,
}

impl DoubleBufferSet {
    pub fn reset_all(&mut self) {
        self.instances.reset();
        self.uniforms.reset();
    }
}

impl DoubleBuffer<QuadInstance> {
    pub fn write_quad(&mut self,render_pass: &mut RenderPass,draw_data: &DrawData) {
        let range = self.push_convert(draw_data.into());
        render_pass.draw_indexed(0..INDEX_BUFFER_SIZE,0,downcast_range(range));
    }
    pub fn write_quad_set(&mut self,render_pass: &mut RenderPass,draw_data: &[DrawData]) {
        let range = self.push_convert_all(draw_data);
        render_pass.draw_indexed(0..INDEX_BUFFER_SIZE,0,downcast_range(range));
    }
}

const fn downcast_range(value: Range<usize>) -> Range<u32> {
    return Range {
        start: value.start as u32,
        end: value.end as u32,
    };
}

struct OutputBuilder {
    encoder: CommandEncoder,
    frame: OutputFrame
}

pub struct GraphicsContext<TConfig> {
    graphics_provider: GraphicsProvider,
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    uniform_bind_group: BindGroup,
    frame_cache: FrameCache<TConfig>,
    output_buffers: DoubleBufferSet,
    output_builder: Option<OutputBuilder>,
}

impl<TConfig> GraphicsContext<TConfig> {
    pub fn get_graphics_provider(&self) -> &GraphicsProvider {
        return &self.graphics_provider;
    }
    pub fn get_graphics_provider_mut(&mut self) -> &mut GraphicsProvider {
        return &mut self.graphics_provider;
    }
}

struct OutputFrame {
    cache_reference: FrameCacheReference,
    surface: SurfaceTexture
}

pub trait GraphicsContextConfig {
    const INSTANCE_CAPACITY: usize;
    const UNIFORM_CAPACITY: usize;
    const CACHE_INSTANCES: usize;
    const CACHE_SIZES: &[(u32,u32)];
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

pub trait GraphicsContextController {
    fn load_texture(&mut self,bytes: &[u8]) -> Frame;
    fn bake(&mut self,frame: &mut Frame) -> Result<(),GraphicsContextError>;
    fn get_frame(&mut self,config: FrameConfig) -> Frame;
}

#[derive(Debug)]
pub enum GraphicsContextError {
    PipelineAlreadyActive,
    PipelineNotActive,
    CantCreateOutputSurface(SurfaceError),
    FrameBakeFailure(FrameError),
    FrameCacheError(CacheArenaError<(u32,u32),FrameCacheReference>)
}

pub trait GraphicsContextInternalController {
    fn create_output_frame(&mut self) -> Result<Frame,GraphicsContextError>;
    fn present_output_frame(&mut self) -> Result<(),GraphicsContextError>;
}

impl<TConfig> GraphicsContextInternalController for GraphicsContext<TConfig>
where
    TConfig: GraphicsContextConfig
{
    fn create_output_frame(&mut self) -> Result<Frame,GraphicsContextError> {
        if self.output_builder.is_some() {
            return Err(GraphicsContextError::PipelineAlreadyActive);
        }

        let surface = match self.graphics_provider.get_output_surface() {
            Ok(value) => value,
            Err(error) => return Err(GraphicsContextError::CantCreateOutputSurface(error)),
        };

        let size = (surface.texture.width(),surface.texture.height());

        let texture_container = TextureContainer::create_output(&surface,size);
        let cache_reference = self.frame_cache.insert_keyless(texture_container);

        self.output_builder = Some(OutputBuilder {
            frame: OutputFrame { cache_reference, surface },
            encoder: self.graphics_provider.get_device().create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder")
            })
        });
        return Ok(FrameInternal::create_output(size,cache_reference));
    }

    fn present_output_frame(&mut self) -> Result<(),GraphicsContextError> {
        let Some(output_builder) = self.output_builder.take() else { //see if there's ANY way to avoid .take() here
            return Err(GraphicsContextError::PipelineNotActive);
        };
        let queue = self.graphics_provider.get_queue();
        self.output_buffers.instances.write_out(queue);
        self.output_buffers.uniforms.write_out_with_padding(queue,UNIFORM_BUFFER_ALIGNMENT);
        queue.submit(std::iter::once(output_builder.encoder.finish()));
        if let Err(error) = self.frame_cache.remove(output_builder.frame.cache_reference) {
            log::warn!("Output frame was not present in the frame cache: {:?}",error);
        };
        output_builder.frame.surface.present();
        self.frame_cache.end_all_leases();
        self.output_buffers.reset_all();
        return Ok(());
    }
}

impl<TConfig> GraphicsContextController for GraphicsContext<TConfig>
where
    TConfig: GraphicsContextConfig
{
    fn load_texture(&mut self,bytes: &[u8]) -> Frame {
        let texture_container = TextureContainer::from_image(
            &self.graphics_provider,
            &self.render_pipeline.get_bind_group_layout(BindGroupIndices::TEXTURE),
            &image::load_from_memory(bytes).unwrap()
        );
        return FrameInternal::create_texture(
            texture_container.size(),
            self.frame_cache.insert_keyless(texture_container)
        );
    }

    fn bake(&mut self,frame: &mut Frame) -> Result<(),GraphicsContextError> {
        let Some(frame_builder) = &mut self.output_builder else {
            frame.clear();
            return Err(GraphicsContextError::PipelineNotActive);
        };

        let texture_view = match self.frame_cache.get(frame.get_cache_reference()) {
            Ok(value) => value.get_view(),
            Err(error) => {
                frame.clear();
                return Err(GraphicsContextError::FrameCacheError(error));
            }
        };

        let mut render_pass = frame_builder.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
        render_pass.set_index_buffer(self.index_buffer.slice(..),wgpu::IndexFormat::Uint32); // Index Buffer
        render_pass.set_vertex_buffer(0,self.vertex_buffer.slice(..)); // Vertex Buffer
        render_pass.set_vertex_buffer(1,self.output_buffers.instances.get_output_buffer().slice(..)); // Instance Buffer

        let uniform_buffer_range = self.output_buffers.uniforms.push(get_camera_uniform(frame.size()));

        let dynamic_offset = uniform_buffer_range.start * UNIFORM_BUFFER_ALIGNMENT;
        render_pass.set_bind_group(BindGroupIndices::UNIFORM,&self.uniform_bind_group,&[dynamic_offset as u32]); // Uniform Buffer Bind Group

        let command_buffer = match frame.get_command_buffer() {
            Ok(value) => value,
            Err(error) => {
                frame.clear();
                return Err(GraphicsContextError::FrameBakeFailure(error));
            },
        };
        process_frame_commands(&self.frame_cache,&mut self.output_buffers.instances,&mut render_pass,command_buffer);
        frame.clear();
        return Ok(());
    }

    fn get_frame(&mut self,config: FrameConfig) -> Frame {
        let frame = match config.lifetime {
            FrameLifetime::Temporary => {
                self.get_temp_frame(config.size,config.draw_once)
            },
            FrameLifetime::Persistent => {
                self.get_persistent_frame(config.size,config.draw_once)
            },
        };
        return frame;
    }
}

impl<TConfig> GraphicsContext<TConfig>
where
    TConfig: GraphicsContextConfig
{
    pub fn create(graphics_provider: GraphicsProvider) -> Self {
        return create_graphics_context(graphics_provider);
    }

    fn get_persistent_frame(&mut self,size: (u32,u32),write_once: bool) -> Frame {
        let size = self.graphics_provider.get_safe_texture_size(size);
        let frame = TextureContainer::create_mutable(
            &self.graphics_provider,
            &self.render_pipeline.get_bind_group_layout(BindGroupIndices::TEXTURE),
            size
        );
        let index = self.frame_cache.insert_keyless(frame);
        return FrameInternal::create(size,FrameCreationOptions {
            persistent: true,
            cache_reference: index,
            write_once
        });
    }

    fn get_temp_frame(&mut self,size: (u32,u32),write_once: bool) -> Frame {
        let size = self.graphics_provider.get_safe_texture_size(size);
        let cache_reference = match self.frame_cache.start_lease(size) {
            Ok(cache_reference) => cache_reference,
            Err(error) => {
                log::info!("Graphics context creating a new temp frame. Reason: {:?}",error);
                let new_texture = TextureContainer::create_mutable(
                    &self.graphics_provider,
                    &self.render_pipeline.get_bind_group_layout(BindGroupIndices::TEXTURE),
                    size
                );
                self.frame_cache.insert_with_lease(size,new_texture)     
            },
        };
        return FrameInternal::create(size,FrameCreationOptions { persistent: false, write_once, cache_reference });
    }
}

fn get_camera_uniform(size: (u32,u32)) -> CameraUniform {
    let (width,height) = size;

    let view_projection = cgmath::ortho(
        0.0, //Left
        width as f32, //Right
        height as f32, //Bottom
        0.0, //Top
        -1.0, //Near
        1.0, //Far
    ).into();

    return CameraUniform { view_projection };
}

fn create_graphics_context<TConfig>(graphics_provider: GraphicsProvider) -> GraphicsContext<TConfig>
where
    TConfig: GraphicsContextConfig
{
    let device = graphics_provider.get_device();
    let render_pipeline = create_wgpu_pipeline(&graphics_provider);
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

    let indices: [u32;INDEX_BUFFER_SIZE as usize] = [
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
        size: (size_of::<QuadInstance>() * TConfig::INSTANCE_CAPACITY) as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let uniform_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("View Projection Buffer"),
        //See: https://docs.rs/wgpu-types/27.0.1/wgpu_types/struct.Limits.html#structfield.min_storage_buffer_offset_alignment
        size: (UNIFORM_BUFFER_ALIGNMENT * TConfig::UNIFORM_CAPACITY) as u64,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false
    });

    let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &render_pipeline.get_bind_group_layout(BindGroupIndices::UNIFORM),
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
        label: Some("View Projection Bind Group"),
    });
    
    let frame_cache = {
        let mut frame_cache = FrameCache::default();
        let cache_instances = TConfig::CACHE_INSTANCES;
        match cache_instances >= 1 {
            true => {
                let bind_group_layout = &render_pipeline.get_bind_group_layout(BindGroupIndices::TEXTURE);
                let mut count = 0;
                for size in TConfig::CACHE_SIZES {
                    let size = graphics_provider.get_safe_texture_size(*size);
                    for _ in 0..cache_instances {
                        let mutable_texture = TextureContainer::create_mutable(&graphics_provider,bind_group_layout,size);
                        frame_cache.insert(size,mutable_texture);
                        count += 1;
                    }
                }
                log::info!("Created {} frame cache instance{}.",count,match count == 1 { true => "", false => "s" });
            },
            false => log::warn!("Frame cache instances is 0. No caches will be created.")
        };
        frame_cache
    };

    let output_buffers = DoubleBufferSet {
        instances: DoubleBuffer::with_capacity(TConfig::INSTANCE_CAPACITY,instance_buffer),
        uniforms: DoubleBuffer::with_capacity(TConfig::UNIFORM_CAPACITY,uniform_buffer),
    };

    return GraphicsContext {
        graphics_provider,
        render_pipeline,
        vertex_buffer,
        index_buffer,
        uniform_bind_group,
        frame_cache,
        output_buffers,
        output_builder: None,
    };
}

fn create_wgpu_pipeline(graphics_provider: &GraphicsProvider) -> RenderPipeline {
    let device = graphics_provider.get_device();

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
                format: graphics_provider.get_output_format(),
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
