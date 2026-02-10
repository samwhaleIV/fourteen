use crate::collections::{
    VecPool,
    cache_arena::CacheArenaError
};

use super::prelude::*;

struct OutputBuilder {
    encoder: CommandEncoder,
    output_frame_reference: FrameCacheReference,
    output_frame_surface: SurfaceTexture,
}

pub struct GraphicsContext {
    graphics_provider: GraphicsProvider,
    pipelines: RenderPipelines,
    frame_cache: FrameCache,
    model_cache: ModelCache,
    output_builder: Option<OutputBuilder>,
    command_buffer_pool: VecPool<FrameCommand,DEFAULT_COMMAND_BUFFER_SIZE>,
    fallback_texture: TextureFrame
}

pub trait GraphicsContextConfig {
    // These are in byte count
    const UNIFORM_BUFFER_SIZE: usize;
    const INSTANCE_BUFFER_SIZE_2D: usize;
    const MODEL_CACHE_VERTEX_BUFFER_SIZE: usize;
    const MODEL_CACHE_INDEX_BUFFER_SIZE: usize;
    const INSTANCE_BUFFER_SIZE_3D: usize;
}

#[derive(Debug)]
pub enum GraphicsContextError {
    PipelineAlreadyActive,
    PipelineNotActive,
    CantCreateOutputSurface(SurfaceError),
    FrameCacheError(CacheArenaError<u32,FrameCacheReference>),
}

impl GraphicsContext {
    pub fn get_graphics_provider(&self) -> &GraphicsProvider {
        return &self.graphics_provider;
    }
    pub fn get_graphics_provider_mut(&mut self) -> &mut GraphicsProvider {
        return &mut self.graphics_provider;
    }
    pub fn create<TConfig: GraphicsContextConfig>(graphics_provider: GraphicsProvider) -> Self {

        let pipelines = RenderPipelines::create::<TConfig>(&graphics_provider);

        let model_cache = ModelCache::create(
            graphics_provider.get_device(),
            TConfig::MODEL_CACHE_VERTEX_BUFFER_SIZE,
            TConfig::MODEL_CACHE_INDEX_BUFFER_SIZE
        );

        let mut graphics_context = Self {
            graphics_provider,
            pipelines,
            model_cache,
            frame_cache: FrameCache::default(),
            command_buffer_pool: VecPool::new(),
            output_builder: None,
            fallback_texture: TextureFrame::placeholder()
        };

        let texture_data = FallbackTexture::create();
        if let Ok(texture_frame) = graphics_context.create_texture_frame(&texture_data) {
            graphics_context.fallback_texture = texture_frame;
        } else {
            log::error!("Could not create a computed fallback texture frame. The fallback texture is set to a fake frame cache reference.");
        }

        return graphics_context;
    }
}

impl GraphicsContext {
    pub fn create_texture_frame(&mut self,texture_data: &impl TextureData) -> Result<TextureFrame,TextureError> {
        let texture_container = TextureContainer::from_image(
            &self.graphics_provider,
            &self.pipelines.shared.texture_layout,
            texture_data
        )?;
        return Ok(FrameFactory::create_texture(
            texture_container.size(),
            self.frame_cache.insert_keyless(texture_container)
        ));
    }

    pub fn get_temp_frame(&mut self,cache_size: CacheSize,clear_color: wgpu::Color) -> TempFrame {
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
    
    pub fn return_temp_frame(&mut self,frame: TempFrame) -> Result<(),GraphicsContextError> {
        let cache_reference = frame.get_cache_reference();

        self.command_buffer_pool.return_item(frame.take_command_buffer());

        if let Err(error) = self.frame_cache.end_lease(cache_reference) {
            return Err(GraphicsContextError::FrameCacheError(error));
        }

        Ok(())
    }

    pub fn get_long_life_frame(&mut self,size: (u32,u32)) -> LongLifeFrame {
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

    pub fn return_long_life_frame(&mut self,frame: LongLifeFrame) -> Result<(),GraphicsContextError> {
        let cache_reference = frame.get_cache_reference();

        self.command_buffer_pool.return_item(frame.take_command_buffer());

        if let Err(error) = self.frame_cache.remove(cache_reference) {
            return Err(GraphicsContextError::FrameCacheError(error));
        }

        Ok(())
    }

    pub fn get_cache_safe_size(&self,size: (u32,u32)) -> CacheSize {
        let output = self.graphics_provider.get_safe_texture_power_of_two(match size.0.max(size.1).checked_next_power_of_two() {
            Some(value) => value,
            None => u32::MAX,
        });
        CacheSize {
            input: size,
            output
        }
    }

    pub fn ensure_frame_for_cache_size(&mut self,cache_size: CacheSize) {
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
    
    pub fn finish_frame_pass<TFrame: MutableFrame,TRenderPass: FrameRenderPass<TFrame>>(&mut self,mut frame_render_pass: TRenderPass) -> Result<TFrame,GraphicsContextError> {
        let frame = frame_render_pass.get_frame_mut();

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

        let mut frame = frame_render_pass.begin_hardware_pass(&mut render_pass,&mut self.pipelines);

        process_frame_commands(
            frame.get_commands(),
            &mut render_pass,
            &mut self.pipelines,
            &self.frame_cache
        );

        frame.clear_commands();
        return Ok(frame);
    }
    
    pub fn get_fallback_texture_frame(&self) -> &TextureFrame {
        &self.fallback_texture
    }

    pub fn create_model_cache_entry(&mut self,gltf_data: &[u8]) -> Result<ModelCacheReference,ModelError> {
        self.model_cache.create_entry(self.graphics_provider.get_queue(),gltf_data)
    }

    pub fn get_render_mesh<'a>(&'a self,model_cache_reference: ModelCacheReference) -> Option<&'a RenderBufferReference> {
        self.model_cache.entries.get(model_cache_reference)
    }

    pub fn get_collision_mesh<'a>(&'a self,model_cache_reference: ModelCacheReference) -> Option<&'a CollisionShape> {
        self.model_cache.collision_shapes.get(model_cache_reference)
    }
}

pub mod swap_chain_control {
    use super::*;
    impl GraphicsContext {
        pub fn create_output_frame(&mut self,clear_color: wgpu::Color) -> Result<OutputFrame,GraphicsContextError> {
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

        pub fn present_output_frame(&mut self,frame: OutputFrame) -> Result<(),GraphicsContextError> {
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
}
