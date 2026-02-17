mod runtime_textures;
mod pipelines;
mod bind_group_cache;

pub use pipelines::*;
pub use bind_group_cache::*;
pub use runtime_textures::*;

use super::prelude::*;

pub struct OutputBuilder<'gc> {
    graphics_context: &'gc mut GraphicsContext,
    encoder: CommandEncoder,
    output_surface: SurfaceTexture,
}

pub struct OutputBuilderContext<'gc> {
    pub builder: OutputBuilder<'gc>,
    pub frame: OutputFrame,
}

pub trait FrameRenderPass<'rp> {
    fn create(
        frame: &impl MutableFrame,
        render_pass: RenderPass<'rp>,
        context: RenderPassContext<'rp>
    ) -> Self;
}

pub struct RenderPassContext<'gc> {
    model_cache: &'gc ModelCache,
    frame_cache: &'gc FrameCache,
    pipelines: &'gc mut RenderPipelines,
    textures: &'gc RuntimeTextures,
    bind_groups: &'gc mut BindGroupCache,
    graphics_provider: &'gc GraphicsProvider
}

pub enum AvailableControls {
    StartOutputFrame,
    RenderPassCreation
}

pub struct GraphicsContext {
    graphics_provider: GraphicsProvider,
    pipelines: RenderPipelines,
    frame_cache: FrameCache,
    runtime_textures: RuntimeTextures,
    model_cache: ModelCache,
    bind_group_cache: BindGroupCache,
    texture_id_generator: TextureIdentityGenerator
}

pub trait GraphicsContextConfig {
    // These are in byte count
    const UNIFORM_BUFFER_SIZE: usize;
    const INSTANCE_BUFFER_SIZE_2D: usize;
    const MODEL_CACHE_VERTEX_BUFFER_SIZE: usize;
    const MODEL_CACHE_INDEX_BUFFER_SIZE: usize;
    const INSTANCE_BUFFER_SIZE_3D: usize;
}

impl GraphicsContext {
    pub fn get_graphics_provider(&self) -> &GraphicsProvider {
        return &self.graphics_provider;
    }
    pub fn get_graphics_provider_mut(&mut self) -> &mut GraphicsProvider {
        return &mut self.graphics_provider;
    }
    pub fn create<TConfig: GraphicsContextConfig>(graphics_provider: GraphicsProvider) -> Self {

        let mut texture_id_generator = TextureIdentityGenerator::default();

        let bind_group_cache = BindGroupCache::create(&graphics_provider);

        let pipelines = RenderPipelines::create::<TConfig>(
            &graphics_provider,
            bind_group_cache.get_texture_layout()
        );

        let model_cache = ModelCache::create(
            graphics_provider.get_device(),
            TConfig::MODEL_CACHE_VERTEX_BUFFER_SIZE,
            TConfig::MODEL_CACHE_INDEX_BUFFER_SIZE
        );

        let mut frame_cache = FrameCache::default();

        let runtime_textures = RuntimeTextures::create(
            &graphics_provider,
            &mut texture_id_generator,
            &mut frame_cache,
        );

        return Self {
            graphics_provider,
            texture_id_generator,
            pipelines,
            model_cache,
            frame_cache,
            runtime_textures,
            bind_group_cache
        };
    }
}

impl GraphicsContext {
    pub fn create_texture_frame(&mut self,texture_data: impl TextureData) -> Result<TextureFrame,TextureError> {
        let texture_id = self.texture_id_generator.next();
        let texture_container = TextureContainer::from_image(
            &self.graphics_provider,
            texture_id,
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
                let texture_id = self.texture_id_generator.next();
                self.frame_cache.insert_with_lease(size,TextureContainer::create_render_target(
                    &self.graphics_provider,
                    texture_id,
                    (size,size)
                ))
            },
        };
        return FrameFactory::create_temp_frame(
            cache_size,
            cache_reference,
            clear_color
        );
    }

    pub fn return_temp_frame(&mut self,frame: TempFrame) -> Result<(),FrameCacheError> {
        let cache_reference = frame.get_cache_reference();
        self.frame_cache.end_lease(cache_reference)?;
        Ok(())
    }

    pub fn get_long_life_frame(&mut self,size: (u32,u32)) -> LongLifeFrame {
        let output = self.graphics_provider.get_safe_texture_size(size);
        let texture_id = self.texture_id_generator.next();
        return FrameFactory::create_long_life(
            RestrictedSize {
                input: size,
                output
            },
            self.frame_cache.insert_keyless(TextureContainer::create_render_target(
                &self.graphics_provider,
                texture_id,
                output
            )),
        );
    }

    pub fn return_long_life_frame(&mut self,frame: LongLifeFrame) -> Result<(),FrameCacheError> {
        let cache_reference = frame.get_cache_reference();
        self.frame_cache.remove(cache_reference)?;
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
        let texture_id = self.texture_id_generator.next();
        self.frame_cache.insert(size,TextureContainer::create_render_target(
            &self.graphics_provider,
            texture_id,
            (size,size)
        ));
    }

    pub fn create_model_cache_entry(&mut self,gltf_data: &[u8]) -> Result<ModelCacheReference,ModelError> {
        self.model_cache.create_entry(self.graphics_provider.get_queue(),gltf_data)
    }

    pub fn get_render_mesh(&self,model_cache_reference: ModelCacheReference) -> Option<RenderBufferReference> {
        self.model_cache.entries.get(model_cache_reference).cloned()
    }

    pub fn get_collision_mesh<'a>(&'a self,model_cache_reference: ModelCacheReference) -> Option<&'a CollisionShape> {
        self.model_cache.collision_shapes.get(model_cache_reference)
    }

    pub fn get_missing_texture(&self) -> TextureFrame {
        return self.runtime_textures.missing.clone();
    }
}

impl GraphicsContext {
    pub fn create_output_builder<'gc>(&'gc mut self,clear_color: WimpyColor) -> Result<OutputBuilderContext<'gc>,SurfaceError> {
        let surface = self.graphics_provider.get_output_surface()?;

        // Note: size is already validated by the graphics provider
        let size = (surface.texture.width(),surface.texture.height());

        let texture_container = TextureContainer::create_output(&surface,size);
        let cache_reference = self.frame_cache.insert_keyless(texture_container);

        let output_frame = FrameFactory::create_output(
            size,
            cache_reference,
            clear_color.into()
        );

        let encoder = self.graphics_provider.get_device().create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder")
        });

        let output_builder = OutputBuilderContext {
            frame: output_frame,
            builder: OutputBuilder {
                output_surface: surface,
                graphics_context: self,
                encoder
            },
        };

        return Ok(output_builder);
    }
}

impl<'gc> OutputBuilder<'gc> {

    fn start_pass<'rp,TRenderPass,TFrame>(
        graphics_context: &'rp mut GraphicsContext,
        encoder: &'rp mut CommandEncoder,
        frame: &'rp TFrame
    ) -> Result<TRenderPass,FrameCacheError>
    where
        TFrame: MutableFrame,
        TRenderPass: FrameRenderPass<'rp>
    {
        let view = graphics_context.frame_cache.get(frame.get_cache_reference())?.get_view();

        let render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
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

        let render_pass_context = RenderPassContext {
            model_cache: &graphics_context.model_cache,
            frame_cache: &graphics_context.frame_cache,
            pipelines: &mut graphics_context.pipelines,
            textures: &graphics_context.runtime_textures,
            bind_groups: &mut graphics_context.bind_group_cache,
            graphics_provider: &graphics_context.graphics_provider
        };

        let frame_render_pass = TRenderPass::create(
            frame,
            render_pass,
            render_pass_context
        );
        return Ok(frame_render_pass);
    }

    pub fn start_pass_2d<'rp,TFrame>(&'rp mut self,frame: &'rp TFrame) -> Result<FrameRenderPass2D<'rp>,FrameCacheError>
    where
        TFrame: MutableFrame
    {
        return Self::start_pass(self.graphics_context,&mut self.encoder,frame);
    }

    pub fn start_pass_3d<'rp,TFrame>(&'rp mut self,frame: &'rp TFrame) -> Result<FrameRenderPass3D<'rp>,FrameCacheError>
    where
        TFrame: MutableFrame
    {
        return Self::start_pass(self.graphics_context,&mut self.encoder,frame);
    }
}

impl OutputBuilderContext<'_> {
    pub fn present_output_surface(self) {
        let graphics_context = self.builder.graphics_context;

        let queue = graphics_context.graphics_provider.get_queue();
        graphics_context.pipelines.write_pipeline_buffers(queue);
        queue.submit(std::iter::once(self.builder.encoder.finish()));

        if let Err(error) = graphics_context.frame_cache.remove(self.frame.get_cache_reference()) {
            log::warn!("Output frame was not present in the frame cache: {:?}",error);
        };
        self.builder.output_surface.present();

        graphics_context.pipelines.reset_pipeline_states();
    }
}
