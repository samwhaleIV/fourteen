mod runtime_textures;
mod pipelines;
mod bind_group_cache;

pub use pipelines::*;
pub use bind_group_cache::*;

use crate::app::{
    WimpyIO,
    wam::AssetManager
};

use super::prelude::*;

pub struct OutputBuilder<'a> {
    graphics_context: &'a mut GraphicsContext,
    encoder: CommandEncoder,
    output_surface: SurfaceTexture,
}

pub struct OutputBuilderContext<'a> {
    pub builder: OutputBuilder<'a>,
    pub frame: OutputFrame,
}

pub trait PipelinePass<'a,'frame> {
    fn create(
        frame: &'frame impl MutableFrame,
        render_pass: &'a mut RenderPass<'frame>,
        context: &'a mut RenderPassContext<'frame>
    ) -> Self;
}

pub struct RenderPassContext<'a> {
    model_cache: &'a ModelCache,
    frame_cache: &'a FrameCache,
    pipelines: &'a mut RenderPipelines,
    textures: &'a EngineTextures,
    bind_groups: &'a mut BindGroupCache,
    graphics_provider: &'a GraphicsProvider
}

pub enum AvailableControls {
    StartOutputFrame,
    RenderPassCreation
}

pub struct EngineTextures {
    pub font_classic: Option<TextureFrame>,
    pub font_classic_outline: Option<TextureFrame>,
    pub font_twelven: Option<TextureFrame>,
    pub font_twelven_shaded: Option<TextureFrame>,

    pub missing: TextureFrame,
    pub opaque_white: TextureFrame,
    pub opaque_black: TextureFrame,
    pub transparent_white: TextureFrame,
    pub transparent_black: TextureFrame,
}

pub struct GraphicsContext {
    graphics_provider: GraphicsProvider,
    pipelines: RenderPipelines,
    frame_cache: FrameCache,
    model_cache: ModelCache,
    bind_group_cache: BindGroupCache,
    texture_id_generator: TextureIdentityGenerator,
    engine_textures: EngineTextures
}

pub trait GraphicsContextConfig {
    // These are in byte count
    const UNIFORM_BUFFER_SIZE: usize;
    const INSTANCE_BUFFER_SIZE_2D: usize;
    const MODEL_CACHE_VERTEX_BUFFER_SIZE: usize;
    const MODEL_CACHE_INDEX_BUFFER_SIZE: usize;
    const INSTANCE_BUFFER_SIZE_3D: usize;
    const TEXT_PIPELINE_BUFFER_SIZE: usize;
}

impl GraphicsContext {
    pub fn get_graphics_provider(&self) -> &GraphicsProvider {
        return &self.graphics_provider;
    }
    pub fn get_graphics_provider_mut(&mut self) -> &mut GraphicsProvider {
        return &mut self.graphics_provider;
    }
    pub async fn create<IO: WimpyIO,TConfig: GraphicsContextConfig>(
        asset_manager: &mut AssetManager,
        graphics_provider: GraphicsProvider
    ) -> Self {

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

        let engine_textures = EngineTextures::create(
            &graphics_provider,
            &mut texture_id_generator,
            &mut frame_cache,
        );

        let mut graphics_context = Self {
            graphics_provider,
            texture_id_generator,
            pipelines,
            model_cache,
            frame_cache,
            engine_textures,
            bind_group_cache,
        };

        graphics_context.load_engine_textures::<IO>(asset_manager).await;

        return graphics_context;
    }

    async fn load_engine_textures<IO: WimpyIO>(&mut self,asset_manager: &mut AssetManager) {
        use assets::*;
        self.engine_textures.font_classic =         self.load_texture::<IO>(asset_manager,FONT_CLASSIC).await;
        self.engine_textures.font_classic_outline = self.load_texture::<IO>(asset_manager,FONT_CLASSIC_OUTLINE).await;
        self.engine_textures.font_twelven =         self.load_texture::<IO>(asset_manager,FONT_TWELVEN).await;
        self.engine_textures.font_twelven_shaded =  self.load_texture::<IO>(asset_manager,FONT_TWELVEN_SHADED).await;
    }

    async fn load_texture<IO: WimpyIO>(&mut self,asset_manager: &mut AssetManager,asset_name: &str) -> Option<TextureFrame> {
        let log_error = |error| {
            log::error!("Engine asset load failure: '{}': {:?}",asset_name,error);
            None
        };
        let key = match asset_manager.get_image_reference(asset_name) {
            Ok(value) => value,
            Err(error) => return log_error(error),
        };
        let image = match asset_manager.load_image::<IO>(&key,self).await {
            Ok(value) => value,
            Err(error) => return log_error(error),
        };
        return Some(image);
    }

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
        let cache_reference = frame.get_ref();
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
        let cache_reference = frame.get_ref();
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
        return self.engine_textures.missing.clone();
    }

    pub fn create_output_builder<'a>(&'a mut self,clear_color: WimpyColor) -> Result<OutputBuilderContext<'a>,SurfaceError> {
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

pub struct RenderPassBuilder<'a,TFrame> {
    frame: &'a TFrame,
    render_pass: RenderPass<'a>,
    context: RenderPassContext<'a>
}

impl<'frame,TFrame> RenderPassBuilder<'frame,TFrame>
where
    TFrame: MutableFrame
{
    pub fn set_pipeline<'a,TPipelinePass>(&'a mut self) -> TPipelinePass
    where
        TPipelinePass: PipelinePass<'a,'frame>
    {
        let pipeline_render_pass = TPipelinePass::create(
            self.frame,
            &mut self.render_pass,
            &mut self.context
        );
        return pipeline_render_pass;
    }

    pub fn set_pipeline_2d<'a>(&'a mut self) -> Pipeline2DPass<'a,'frame> {
        self.set_pipeline()
    }

    pub fn set_pipeline_3d<'a>(&'a mut self) -> Pipeline3DPass<'a,'frame> {
        self.set_pipeline()
    }

    pub fn set_pipeline_text<'a>(&'a mut self) -> PipelineTextPass<'a,'frame> {
        self.set_pipeline()
    }
}

//experiment where we put 'a
impl OutputBuilder<'_> {

    pub fn create_render_pass<'a,TFrame>(&'a mut self,frame: &'a TFrame) -> Result<RenderPassBuilder<'a,TFrame>,FrameCacheError>
    where
        TFrame: MutableFrame
    {
        let view = self.graphics_context.frame_cache.get(frame.get_ref())?.get_view();

        let render_pass = self.encoder.begin_render_pass(&RenderPassDescriptor {
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

        let context = RenderPassContext {
            model_cache: &self.graphics_context.model_cache,
            frame_cache: &self.graphics_context.frame_cache,
            pipelines: &mut self.graphics_context.pipelines,
            textures: &self.graphics_context.engine_textures,
            bind_groups: &mut self.graphics_context.bind_group_cache,
            graphics_provider: &self.graphics_context.graphics_provider,
        };

        return Ok(RenderPassBuilder {
            frame,
            render_pass,
            context,
        })
    }
}

impl OutputBuilderContext<'_> {
    pub fn present_output_surface(self) {
        let graphics_context = self.builder.graphics_context;

        let queue = graphics_context.graphics_provider.get_queue();
        graphics_context.pipelines.write_pipeline_buffers(queue);
        queue.submit(std::iter::once(self.builder.encoder.finish()));

        if let Err(error) = graphics_context.frame_cache.remove(self.frame.get_ref()) {
            log::warn!("Output frame was not present in the frame cache: {:?}",error);
        };
        self.builder.output_surface.present();

        graphics_context.pipelines.reset_pipeline_states();
    }
}
