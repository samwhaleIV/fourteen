use wgpu::*;

use crate::{
    shared::*,
    wgpu::{
        FrameCommand,
        command_processor::process_frame_commands,
        frame::*,
        frame_cache::*,
        graphics_provider::GraphicsProvider,
        pipelines::*,
        texture_container::*
    }
};

pub const DEFAULT_COMMAND_BUFFER_SIZE: usize = 32;

struct OutputBuilder {
    encoder: CommandEncoder,
    output_frame_reference: FrameCacheReference,
    output_frame_surface: SurfaceTexture,
}

pub struct GraphicsContext<TConfig> {
    graphics_provider: GraphicsProvider,
    pipelines: RenderPipelines,
    frame_cache: FrameCache<TConfig>,
    output_builder: Option<OutputBuilder>, //Technically a finite state machine
    command_buffer_pool: VecPool<FrameCommand,DEFAULT_COMMAND_BUFFER_SIZE>,
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
    fn create_texture_frame(&mut self,texture_data: impl TextureData) -> Result<TextureFrame,GraphicsContextError>;

    fn render_frame_2D(&mut self,frame: &mut impl MutableFrame) -> Result<(),GraphicsContextError>;
    fn render_frame_3D(&mut self,frame: &mut impl MutableFrame) -> Result<(),GraphicsContextError>;

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
    FrameCacheError(CacheArenaError<u32,FrameCacheReference>),
    TextureCreationFailure(TextureContainerError)
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
        
        // Investigate: only finalize the pipelines that were used during this output builder's life (or let the pipelines no-op on their own?)
        self.pipelines.pipeline_2d.write_buffers(queue);
        self.pipelines.pipeline_3d.write_buffers(queue);
        
        // We always write the shared buffers
        self.pipelines.shared.write_buffers(queue);

        queue.submit(std::iter::once(output_builder.encoder.finish()));
        if let Err(error) = self.frame_cache.remove(output_builder.output_frame_reference) {
            log::warn!("Output frame was not present in the frame cache: {:?}",error);
        };
        output_builder.output_frame_surface.present();
        
        self.pipelines.pipeline_2d.reset_buffers();
        self.pipelines.pipeline_3d.reset_buffers();

        self.pipelines.shared.reset_buffers();

        self.command_buffer_pool.return_item(frame.take_command_buffer());
        return Ok(());
    }
}

impl<TConfig> GraphicsContextController for GraphicsContext<TConfig>
where
    TConfig: GraphicsContextConfig
{
    fn create_texture_frame(&mut self,texture_data: impl TextureData) -> Result<TextureFrame,GraphicsContextError> {
        let texture_container = match TextureContainer::from_image(
            &self.graphics_provider,
            &self.pipelines.shared.texture_layout,
            texture_data
        ) {
            Ok(value) => value,
            Err(error) => return Err(GraphicsContextError::TextureCreationFailure(error))
        };
        return Ok(FrameFactory::create_texture(
            texture_container.size(),
            self.frame_cache.insert_keyless(texture_container)
        ));
    }

    fn get_temp_frame(&mut self,cache_size: CacheSize,clear_color: wgpu::Color) -> TempFrame {
        let size = cache_size.output;
        let cache_reference = match self.frame_cache.start_lease(size) {
            Ok(value) => value, 
            Err(error) => {
                log::warn!("Graphics context creating a new temp frame. Reason: {:?}",error);
                self.frame_cache.insert_with_lease(size,TextureContainer::create_mutable(
                    &self.graphics_provider,
                    &self.pipelines.shared.texture_layout,
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
                &self.pipelines.shared.texture_layout,
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
            &self.pipelines.shared.texture_layout,
            (size,size)
        ));
    }
    
    fn render_frame_2D(&mut self,frame: &mut impl MutableFrame) -> Result<(),GraphicsContextError> {
        let camera_uniform = CameraUniform::create_ortho(frame.get_input_size());
        return self.render_frame::<Pipeline2D>(frame,camera_uniform);
    }
    
    fn render_frame_3D(&mut self,frame: &mut impl MutableFrame) -> Result<(),GraphicsContextError> {
        let camera_uniform = CameraUniform::placeholder(); //TODO
        return self.render_frame::<Pipeline3D>(frame,camera_uniform);
    }
}

impl<TConfig> GraphicsContext<TConfig>
where
    TConfig: GraphicsContextConfig
{
    pub fn create(graphics_provider: GraphicsProvider) -> Self {

        let render_pipelines = RenderPipelines::create::<TConfig>(&graphics_provider);

        return Self {
            graphics_provider,
            pipelines: render_pipelines,
            frame_cache: FrameCache::default(),
            command_buffer_pool: VecPool::new(),
            output_builder: None,
        }
    }

    fn render_frame<TPipelineSelector>(
        &mut self,
        frame: &mut impl MutableFrame,
        uniform: CameraUniform,
    ) -> Result<(),GraphicsContextError>
    where
        TPipelineSelector: RenderPassController
    {
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

        TPipelineSelector::select_and_begin(
            &mut render_pass,
            &mut self.pipelines,
            uniform
        );

        process_frame_commands(
            frame.get_commands(),
            &mut render_pass,
            &mut self.pipelines,
            &self.frame_cache
        );

        frame.clear_commands();
        return Ok(());
    }
}
