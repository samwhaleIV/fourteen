use std::u32;

use wgpu::{
    BindGroup,
    BindGroupLayoutDescriptor,
    Buffer,
    BufferDescriptor,
    BufferUsages,
    CommandEncoder,
    CommandEncoderDescriptor,
    RenderPipeline,
    SurfaceError,
    SurfaceTexture,
    util::{
        BufferInitDescriptor,
        DeviceExt
    }
};

use crate::{
    shared::{
        CacheArenaError,
        VecPool
    },
    wgpu::{
        command_processor::process_frame_commands,
        constants::{
            BindGroupIndices,
            DEFAULT_COMMAND_BUFFER_SIZE,
            INDEX_BUFFER_SIZE,
            UNIFORM_BUFFER_ALIGNMENT
        }, double_buffer::DoubleBuffer, double_buffer_set::DoubleBufferSet, frame_cache::{
            FrameCache,
            FrameCacheReference
        }, shader_definitions::{
            CameraUniform,
            QuadInstance,
            Vertex
        }, texture_container::TextureData
    }
};

use super::{
    texture_container::TextureContainer,
    graphics_provider::GraphicsProvider,
    frame::*
};

struct OutputBuilder {
    encoder: CommandEncoder,
    output_frame_reference: FrameCacheReference,
    output_frame_surface: SurfaceTexture,
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
    command_buffer_pool: VecPool<FrameCommand,DEFAULT_COMMAND_BUFFER_SIZE>
}

impl<TConfig> GraphicsContext<TConfig> {
    pub fn get_graphics_provider(&self) -> &GraphicsProvider {
        return &self.graphics_provider;
    }
    pub fn get_graphics_provider_mut(&mut self) -> &mut GraphicsProvider {
        return &mut self.graphics_provider;
    }
}

pub trait GraphicsContextConfig {
    const INSTANCE_CAPACITY: usize;
    const UNIFORM_CAPACITY: usize;
}

pub trait GraphicsContextController {
    fn create_texture_frame(&mut self,texture_data: impl TextureData) -> TextureFrame;
    fn render_frame(&mut self,frame: &mut impl MutableFrame) -> Result<(),GraphicsContextError>;

    fn get_cache_safe_size(&self,size: (u32,u32)) -> CacheSize;
    fn ensure_frame_for_cache_size(&mut self,cache_size: CacheSize);

    fn get_temp_frame(&mut self,cache_size: CacheSize,clear_color: wgpu::Color) -> TempFrame;
    fn return_temp_frame(&mut self,frame: TempFrame) -> Result<(),GraphicsContextError>;

    fn create_long_life_frame(&mut self,size: (u32,u32)) -> LongLifeFrame;
}

#[derive(Debug)]
pub enum GraphicsContextError {
    PipelineAlreadyActive,
    PipelineNotActive,
    CantCreateOutputSurface(SurfaceError),
    FrameCacheError(CacheArenaError<u32,FrameCacheReference>)
}

pub trait GraphicsContextInternalController {
    fn create_output_frame(&mut self,clear_color: wgpu::Color) -> Result<OutputFrame,GraphicsContextError>;
    fn present_output_frame(&mut self,frame: OutputFrame) -> Result<(),GraphicsContextError>;
}

impl<TConfig> GraphicsContextInternalController for GraphicsContext<TConfig>
where
    TConfig: GraphicsContextConfig
{
    fn create_output_frame(&mut self,clear_color: wgpu::Color) -> Result<OutputFrame,GraphicsContextError> {
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
            output_frame_reference: cache_reference,
            output_frame_surface: surface,
            encoder: self.graphics_provider.get_device().create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder")
            })
        });

        return Ok(FrameFactory::create_output(
            size,
            cache_reference,
            self.command_buffer_pool.take_item(),
            clear_color
        ));
    }

    fn present_output_frame(&mut self,frame: OutputFrame) -> Result<(),GraphicsContextError> {
        let Some(output_builder) = self.output_builder.take() else { //see if there's ANY way to avoid .take() here
            return Err(GraphicsContextError::PipelineNotActive);
        };
        let queue = self.graphics_provider.get_queue();
        self.output_buffers.instances.write_out(queue);
        self.output_buffers.uniforms.write_out_with_padding(queue,UNIFORM_BUFFER_ALIGNMENT);
        queue.submit(std::iter::once(output_builder.encoder.finish()));
        if let Err(error) = self.frame_cache.remove(output_builder.output_frame_reference) {
            log::warn!("Output frame was not present in the frame cache: {:?}",error);
        };
        output_builder.output_frame_surface.present();
        self.output_buffers.reset_all();
        self.command_buffer_pool.return_item(frame.take_command_buffer());
        return Ok(());
    }
}

impl<TConfig> GraphicsContextController for GraphicsContext<TConfig>
where
    TConfig: GraphicsContextConfig
{
    fn create_texture_frame(&mut self,texture_data: impl TextureData) -> TextureFrame {
        let texture_container = TextureContainer::from_image(
            &self.graphics_provider,
            &self.render_pipeline.get_bind_group_layout(BindGroupIndices::TEXTURE),
            texture_data
        );
        return FrameFactory::create_texture(
            texture_container.size(),
            self.frame_cache.insert_keyless(texture_container)
        );
    }

    fn render_frame(&mut self,frame: &mut impl MutableFrame) -> Result<(),GraphicsContextError> {
        let Some(frame_builder) = &mut self.output_builder else {
            frame.clear_commands();
            return Err(GraphicsContextError::PipelineNotActive);
        };

        let texture_view = match self.frame_cache.get(frame.get_cache_reference()) {
            Ok(value) => value.get_view(),
            Err(error) => {
                frame.clear_commands();
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

        let uniform_buffer_range = self.output_buffers.uniforms.push(get_camera_uniform(frame.get_input_size()));

        let dynamic_offset = uniform_buffer_range.start * UNIFORM_BUFFER_ALIGNMENT;
        render_pass.set_bind_group(BindGroupIndices::UNIFORM,&self.uniform_bind_group,&[dynamic_offset as u32]); // Uniform Buffer Bind Group

        process_frame_commands(
            &self.frame_cache,
            &mut self.output_buffers.instances,
            &mut render_pass,frame.get_commands()
        );
        frame.clear_commands();
        return Ok(());
    }

    fn get_temp_frame(&mut self,cache_size: CacheSize,clear_color: wgpu::Color) -> TempFrame {
        let size = cache_size.output;
        let cache_reference = match self.frame_cache.start_lease(size) {
            Ok(value) => value, 
            Err(error) => {
                log::warn!("Graphics context creating a new temp frame. Reason: {:?}",error);
                self.frame_cache.insert_with_lease(size,TextureContainer::create_mutable(
                    &self.graphics_provider,
                    &self.render_pipeline.get_bind_group_layout(BindGroupIndices::TEXTURE),
                    (size,size)
                ))
            },
        };
        return FrameFactory::create_temp_frame(
            cache_size,
            cache_reference,
            self.command_buffer_pool.take_item(),
            clear_color
        );
    }
    
    fn return_temp_frame(&mut self,frame: TempFrame) -> Result<(),GraphicsContextError> {
        let cache_reference = frame.get_cache_reference();

        self.command_buffer_pool.return_item(frame.take_command_buffer());

        if let Err(error) = self.frame_cache.end_lease(cache_reference) {
            return Err(GraphicsContextError::FrameCacheError(error));
        }

        Ok(())
    }

    fn create_long_life_frame(&mut self,size: (u32,u32)) -> LongLifeFrame {
        let output = self.graphics_provider.get_safe_texture_size(size);
        return FrameFactory::create_long_life(
            RestrictedSize {
                input: size,
                output
            },
            self.frame_cache.insert_keyless(TextureContainer::create_mutable(
                &self.graphics_provider,
                &self.render_pipeline.get_bind_group_layout(BindGroupIndices::TEXTURE),
                output
            )),
            Vec::with_capacity(DEFAULT_COMMAND_BUFFER_SIZE)
        );
    }

    fn get_cache_safe_size(&self,size: (u32,u32)) -> CacheSize {
        let output = self.graphics_provider.get_safe_texture_power_of_two(match size.0.max(size.1).checked_next_power_of_two() {
            Some(value) => value,
            None => u32::MAX,
        });
        CacheSize {
            input: size,
            output
        }
    }

    fn ensure_frame_for_cache_size(&mut self,cache_size: CacheSize) {
        let size = cache_size.output;
        if self.frame_cache.has_available_items(size) {
            return;
        }
        self.frame_cache.insert(size,TextureContainer::create_mutable(
            &self.graphics_provider,
            &self.render_pipeline.get_bind_group_layout(BindGroupIndices::TEXTURE),
            (size,size)
        ));
    }
}

impl<TConfig> GraphicsContext<TConfig>
where
    TConfig: GraphicsContextConfig
{
    pub fn create(graphics_provider: GraphicsProvider) -> Self {
        return create_graphics_context(graphics_provider);
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

    let output_buffers = DoubleBufferSet {
        instances: DoubleBuffer::with_capacity(TConfig::INSTANCE_CAPACITY,instance_buffer),
        uniforms: DoubleBuffer::with_capacity(TConfig::UNIFORM_CAPACITY,uniform_buffer),
    };

    let frame_cache = FrameCache::default();
    let command_buffer_pool = VecPool::new();

    return GraphicsContext {
        graphics_provider,
        render_pipeline,
        vertex_buffer,
        index_buffer,
        uniform_bind_group,
        frame_cache,
        output_buffers,
        command_buffer_pool,
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
