use super::{*,pipelines::core::*};
use wgpu::*;

use crate::world::{CameraPerspectivePacket, WimpyCamera};
use crate::{UWimpyPoint, WimpyColor, WimpyVec};
use crate::app::wam::AssetManager;

use pipelines::{
    pipeline_2d::*,
    pipeline_3d::*,
    text_pipeline::*,
    lines_pipeline::*,
};

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
        render_pass: &'a mut RenderPass<'frame>,
        context: &'a mut RenderPassContext<'frame>,
        uniform_reference: UniformReference,
    ) -> Self;
}

pub struct RenderPassContext<'a> {
    pub variant_key: PipelineVariantKey,
    pub model_cache: &'a ModelCache,
    pub frame_cache: &'a FrameCache,
    pub pipelines: &'a mut RenderPipelines,
    pub textures: &'a EngineTextures,
    pub bind_groups: &'a mut BindGroupCache,
    pub graphics_provider: &'a GraphicsProvider
}

pub enum AvailableControls {
    StartOutputFrame,
    RenderPassCreation
}

pub struct EngineTextures {
    pub font_classic: TextureFrame,
    pub font_classic_outline: TextureFrame,
    pub font_twelven: TextureFrame,
    pub font_twelven_shaded: TextureFrame,
    pub font_mono_elf: TextureFrame,

    pub missing: TextureFrame,
    pub opaque_white: TextureFrame,
    pub opaque_black: TextureFrame,
    pub transparent_white: TextureFrame,
    pub transparent_black: TextureFrame,
}

pub struct GraphicsContext {
    pub graphics_provider: GraphicsProvider,
    pub pipelines: RenderPipelines,
    pub frame_cache: FrameCache,
    pub model_cache: ModelCache,
    pub bind_groups: BindGroupCache,
    pub texture_id_generator: TextureIdentityGenerator,
    pub engine_textures: EngineTextures
}

pub trait GraphicsContextConfig {
    // These are in byte count
    const UNIFORM_BUFFER_SIZE: usize;
    const INSTANCE_BUFFER_SIZE_2D: usize;
    const MODEL_CACHE_VERTEX_BUFFER_SIZE: usize;
    const MODEL_CACHE_INDEX_BUFFER_SIZE: usize;
    const INSTANCE_BUFFER_SIZE_3D: usize;
    const TEXT_PIPELINE_BUFFER_SIZE: usize;
    const LINE_BUFFER_SIZE: usize;
}

#[derive(Copy,Clone)]
pub struct CameraPerspective {
    pub fov: f32,
    pub clip_near: f32,
    pub clip_far: f32,
}

impl Default for CameraPerspective {
    fn default() -> Self {
        Self {
            clip_near: 0.025,
            clip_far: 100.0,
            fov: 90.0
        }
    }
}

impl GraphicsContext {
    pub fn get_graphics_provider(&self) -> &GraphicsProvider {
        return &self.graphics_provider;
    }

    pub fn get_graphics_provider_mut(&mut self) -> &mut GraphicsProvider {
        return &mut self.graphics_provider;
    }

    pub async fn create<TConfig>(graphics_provider: GraphicsProvider) -> Self
    where
        TConfig: GraphicsContextConfig
    {
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

        Self {
            graphics_provider,
            texture_id_generator,
            pipelines,
            model_cache,
            frame_cache,
            engine_textures,
            bind_groups: bind_group_cache,
        }
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
        let size = cache_size.output_single_dimension;
        let cache_reference = match self.frame_cache.start_lease(size) {
            Ok(value) => value, 
            Err(error) => {
                log::warn!("Graphics context creating a new temp frame. Reason: {:?}",error);
                let texture_id = self.texture_id_generator.next();
                self.frame_cache.insert_with_lease(size,TextureContainer::create_render_target(
                    &self.graphics_provider,
                    texture_id,
                    size.into()
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

    pub fn get_long_life_frame(&mut self,size: UWimpyPoint) -> LongLifeFrame {
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

    pub fn get_cache_safe_size(&self,size: UWimpyPoint) -> CacheSize {
        let output = self.graphics_provider.get_safe_texture_power_of_two(match size.largest().checked_next_power_of_two() {
            Some(value) => value,
            None => u32::MAX,
        });
        CacheSize {
            input: size,
            output_single_dimension: output
        }
    }

    pub fn ensure_frame_for_cache_size(&mut self,cache_size: CacheSize) {
        let size = cache_size.output_single_dimension;
        if self.frame_cache.has_available_items(size) {
            return;
        }
        let texture_id = self.texture_id_generator.next();
        self.frame_cache.insert(size,TextureContainer::create_render_target(
            &self.graphics_provider,
            texture_id,
            size.into()
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

    pub fn create_output_builder<'a>(&'a mut self,color: impl WimpyColor) -> Option<OutputBuilderContext<'a>> {
        let output_surface = match self.graphics_provider.get_output_surface() {
            Ok(value) => value,
            Err(error) => {
                log::error!("Could not create output surface: {:?}",error);
                return None;
            },
        };

        // Note: size is already validated by the graphics provider
        let size: UWimpyPoint = [output_surface.texture.width(),output_surface.texture.height()].into();

        let texture_container = TextureContainer::create_output(
            &output_surface,
            self.graphics_provider.get_output_view_format(),
            size
        );

        let cache_reference = self.frame_cache.insert_keyless(texture_container);

        let encoder = self.graphics_provider.get_device().create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder")
        });

        let frame = FrameFactory::create_output(
            size,
            cache_reference,
            color.into_linear().into()
        );

        let output_builder = OutputBuilderContext {
            builder: OutputBuilder {
                graphics_context: self,
                encoder,
                output_surface
            },
            frame,
        };

        return Some(output_builder);
    }
}

pub struct RenderPassBuilder<'a,TFrame> {
    frame: &'a TFrame,
    ortho_uniform: UniformReference,
    render_pass: RenderPass<'a>,
    context: RenderPassContext<'a>
}

impl<'frame,TFrame> RenderPassBuilder<'frame,TFrame>
where
    TFrame: MutableFrame
{
    fn set_pipeline<'a,TPipelinePass>(&'a mut self,uniform_reference: UniformReference) -> TPipelinePass
    where
        TPipelinePass: PipelinePass<'a,'frame>
    {
        let pipeline_render_pass = TPipelinePass::create(
            &mut self.render_pass,
            &mut self.context,
            uniform_reference
        );
        return pipeline_render_pass;
    }

    pub fn set_pipeline_2d<'a>(&'a mut self) -> Pipeline2DPass<'a,'frame> {
        self.set_pipeline(self.ortho_uniform)
    }

    pub fn set_pipeline_3d<'a>(&'a mut self,uniform_reference: UniformReference) -> Pipeline3DPass<'a,'frame> {
        self.set_pipeline(uniform_reference)
    }

    pub fn set_pipeline_text<'a,TFont: FontDefinition>(&'a mut self) -> PipelineTextPass<'a,'frame,TFont> {
        self.set_pipeline(self.ortho_uniform)
    }

    pub fn set_pipeline_lines_2d<'a>(&'a mut self) -> LinesPipelinePass<'a,'frame> {
        self.set_pipeline(self.ortho_uniform)
    }

    pub fn set_pipeline_lines_3d<'a>(&'a mut self,uniform_reference: UniformReference) -> LinesPipelinePass<'a,'frame> {
        self.set_pipeline(uniform_reference)
    }

    pub fn create_camera_uniform(
        &mut self,
        camera: &WimpyCamera,
        config: CameraPerspective,
    ) -> UniformReference {
        let matrix = camera.get_matrix(CameraPerspectivePacket {
            fov: config.fov,
            clip_near: config.clip_near,
            clip_far: config.clip_far,
            aspect_ratio: self.frame.aspect_ratio(),
        });
        self.context.pipelines.get_shared_mut().create_uniform(matrix)
    }

    pub fn frame(&self) -> &TFrame {
        return self.frame;
    }
}

impl OutputBuilder<'_> {

    pub fn create_render_pass<'a,TFrame>(&'a mut self,frame: &'a TFrame) -> Result<RenderPassBuilder<'a,TFrame>,FrameCacheError>
    where
        TFrame: MutableFrame
    {
        let view = self.graphics_context.frame_cache.get(frame.get_ref())?.get_view();

        let mut render_pass = self.encoder.begin_render_pass(&RenderPassDescriptor {
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

        let frame_size = WimpyVec::from(frame.get_input_size());
        render_pass.set_viewport(0.0,0.0,frame_size.x,frame_size.y,0.0,1.0);

        let pipeline_variant = match frame.is_output_surface() {
            true => PipelineVariantKey::OutputSurface,
            false => PipelineVariantKey::InternalTarget,
        };

        let mut context = RenderPassContext {
            model_cache: &self.graphics_context.model_cache,
            frame_cache: &self.graphics_context.frame_cache,
            pipelines: &mut self.graphics_context.pipelines,
            textures: &self.graphics_context.engine_textures,
            bind_groups: &mut self.graphics_context.bind_groups,
            graphics_provider: &self.graphics_context.graphics_provider,
            variant_key: pipeline_variant,
        };

        let ortho_uniform = context.get_shared_mut().create_uniform_ortho(frame.size());

        return Ok(RenderPassBuilder {
            frame,
            render_pass,
            context,
            ortho_uniform,
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
