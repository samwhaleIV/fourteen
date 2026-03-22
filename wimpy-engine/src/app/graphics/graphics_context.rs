use wgpu::*;
use crate::{UWimpyPoint, WimpyColor, WimpyVec, app::fonts::FontDefinition, world::{Frustum, WimpyCamera}};
use super::{*, textures::*, pipelines::*};

pub struct OutputBuilder<'a> {
    graphics_context: &'a mut GraphicsContext,
    encoder: CommandEncoder,
    output_surface: SurfaceTexture,
}

pub struct OutputBuilderContext<'a> {
    pub builder: OutputBuilder<'a>,
    pub frame: OutputRenderTarget,
}

pub enum AvailableControls {
    StartOutpuTRenderTarget,
    RenderPassCreation
}

pub struct GraphicsContext {
    pub graphics_provider:  GraphicsProvider,
    pub pipelines:          RenderPipelines,
    pub texture_manager:    TextureManager,
    pub mesh_cache:         MeshCache,
    ///A depth stencil exclusively for the output surface
    /// 
    /// This avoids possible churn when using render targets that use depth stencil render passes
    output_depth_stencil:   Option<DepthStencil>,
    depth_stencil:          Option<DepthStencil>
}

struct DepthStencil {
    texture: Texture,
    view: TextureView,
}

impl DepthStencil {
    fn create(device: &Device,size: UWimpyPoint) -> Self {
        let size = Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        };
        let descriptor = TextureDescriptor {
            label: Some("Depth Stencil Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: constants::DEPTH_STENCIL_TEXTURE_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        };

        let texture = device.create_texture(&descriptor);
        let view = texture.create_view(&TextureViewDescriptor::default());

        Self {
            texture,
            view,
        }
    }
}

impl GraphicsContext {
    pub async fn create<TConfig>(graphics_provider: GraphicsProvider,streaming_policy: StreamingPolicy) -> Self
    where
        TConfig: GraphicsConfig
    {
        let pipeline_core = PipelineCore::create::<TConfig>(&graphics_provider);

        let mut texture_manager = TextureManager::new(
            &graphics_provider,
            pipeline_core.texture_layout.clone(),
            streaming_policy
        );

        let mut mesh_cache = MeshCache::create(
            graphics_provider.get_device(),
            TConfig::MESH_CACHE_VERTEX_BUFFER_SIZE,
            TConfig::MESH_CACHE_INDEX_BUFFER_SIZE
        );

        let pipelines = RenderPipelines::create::<TConfig>(
            &graphics_provider,
            &mut texture_manager,
            &mut mesh_cache,
            pipeline_core,
        );

        Self {
            graphics_provider,
            pipelines,
            texture_manager,
            mesh_cache,
            output_depth_stencil: None,
            depth_stencil: None,
        }
    }

    pub fn get_temp_frame(&mut self,size: UWimpyPoint,clear_color: Color) -> TempRenderTarget {
        let cache_key = self.graphics_provider.get_safe_texture_power_of_two(match size.largest().checked_next_power_of_two() {
            Some(value) => value,
            None => u32::MAX,
        });

        let output_size: UWimpyPoint = cache_key.into();
        let gpu_texture_key = self.texture_manager.borrow_render_target(&self.graphics_provider,output_size,cache_key);

        TempRenderTarget::new(
            FilteredSize {
                input: size,
                output: output_size
            },
            gpu_texture_key,
            clear_color
        )
    }

    pub fn return_temp_frame(&mut self,frame: TempRenderTarget) -> Result<(),GPUTextureCacheError> {
        let texture_key = frame.get_key();
        self.texture_manager.gpu_cache.end_lease(texture_key)?;
        Ok(())
    }

    pub fn get_long_life_frame(&mut self,size: UWimpyPoint) -> LongLifeRenderTarget {
        let output_size = self.graphics_provider.get_safe_texture_size(size);
        let gpu_texture_key = self.texture_manager.create_keyless_render_target(&self.graphics_provider,output_size);
        LongLifeRenderTarget::new(
            FilteredSize {
                input: size,
                output: output_size
            },
            gpu_texture_key,
        )
    }

    pub fn return_long_life_frame(&mut self,frame: LongLifeRenderTarget) -> Result<(),GPUTextureCacheError> {
        let texture_key = frame.get_key();
        self.texture_manager.gpu_cache.remove(texture_key)?;
        Ok(())
    }

    /// Preallocate a GPU texture for usage as a render target of this size if none exist or they are all presently leased
    pub fn ensure_temp_frame_for_size(&mut self,size: UWimpyPoint) {
        let cache_key = self.graphics_provider.get_safe_texture_power_of_two(match size.largest().checked_next_power_of_two() {
            Some(value) => value,
            None => u32::MAX,
        });
        self.texture_manager.ensure_cached_render_target(&self.graphics_provider,size,cache_key)
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
        let view_format = self.graphics_provider.get_output_view_format();

        let cache_reference = self.texture_manager.bind_output_surface(&output_surface,view_format,size);

        let encoder = self.graphics_provider.get_device().create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder")
        });

        let frame = OutputRenderTarget::new(
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

        Some(output_builder)
    }
}

pub struct RenderPassBuilder<'a,TRenderTarget> {
    render_pass: RenderPass<'a>,
    context: &'a mut GraphicsContext,
    frame: &'a TRenderTarget,
    ortho_uniform: UniformReference,
    variant_key: PipelineVariantKey,
}

impl<'context,TRenderTarget> RenderPassBuilder<'context,TRenderTarget>
where
    TRenderTarget: RenderTarget
{
    fn set_pipeline<'pass,TPipelinePass>(&'pass mut self,uniform_reference: UniformReference) -> TPipelinePass
    where
        TPipelinePass: PipelinePass<'pass,'context>
    {
        TPipelinePass::create(
            &mut self.render_pass,
            &mut self.context,
            self.variant_key,
            uniform_reference
        )
    }

    pub fn set_pipeline_2d(&mut self) -> Pipeline2DPass<'_,'context> { self.set_pipeline(self.ortho_uniform) }

    pub fn set_pipeline_text<TFont: FontDefinition>(&mut self) -> PipelineTextPass<'_,'context,TFont> { self.set_pipeline(self.ortho_uniform) }

    pub fn set_pipeline_lines_2d(&mut self) -> LinesPipelinePass<'_,'context> { self.set_pipeline(self.ortho_uniform) }

    pub fn set_pipeline_lines_3d(&mut self,uniform: UniformReference) -> LinesPipelinePass<'_,'context> { self.set_pipeline(uniform) }

    pub fn create_camera_uniform(
        &mut self,
        camera: &WimpyCamera,
        frustum: Frustum,
    ) -> UniformReference {
        let matrix = camera.get_matrix(frustum,self.frame.aspect_ratio());
        self.context.pipelines.core.create_uniform(matrix)
    }

    /// Safe to call even if no meshes have been submitted, there is an early exit path
    /// 
    /// `output.builder.submit_batched_meshes()` must be called before this render pass was created
    pub fn draw_submitted_meshes(&mut self,diffuse_sampler: SamplerMode,uniform: UniformReference) {
        let mut pipeline_3d_pass = self.set_pipeline::<Pipeline3DPass>(uniform);
        pipeline_3d_pass.submit(diffuse_sampler);
    }

    pub fn frame(&self) -> &TRenderTarget { self.frame }
}

#[derive(Copy,Clone)]
enum DepthStencilConfig {
    None,
    Standard
}

impl OutputBuilder<'_> {
    /// Batch meshes to prepare an encoder submission. This must happen before a render pass is created
    /// 
    /// Before the render pass, `submit_batched_meshes()` must be called. Later, `render_pass.draw_submitted_meshes()` can be used during an active render pass
    pub fn batch_meshes<'a,I>(&'a mut self,texture_strategy: TextureStrategy,draw_data: I)
    where
        I: IntoIterator<Item = DrawData3D>
    {
        Pipeline3D::batch(self.graphics_context,texture_strategy,draw_data);
    }

    /// Must be called before the first render pass that will draw meshes executes
    pub fn submit_batched_meshes(&mut self) {
        self.graphics_context.pipelines.pipeline_3d.flush_encoder(&mut self.encoder);
    }

    fn create_render_pass_internal<'a,TRenderTarget>(&'a mut self,frame: &'a TRenderTarget,depth_stencil_config: DepthStencilConfig) -> Result<RenderPassBuilder<'a,TRenderTarget>,GPUTextureCacheError>
    where
        TRenderTarget: RenderTarget
    {
        let view = frame.get_cache_entry(&mut self.graphics_context.texture_manager).value.get_view();

        let pipeline_variant = match (frame.is_output_surface(),depth_stencil_config) {
            (true, DepthStencilConfig::None) =>         PipelineVariantKey::OutputSurface,
            (true, DepthStencilConfig::Standard) =>     PipelineVariantKey::OutputSurfaceWithDepth,
            (false, DepthStencilConfig::None) =>        PipelineVariantKey::RenderTarget,
            (false, DepthStencilConfig::Standard) =>    PipelineVariantKey::InternalTargetWithDepth,
        };

        let depth_stencil_attachment = match depth_stencil_config {
            DepthStencilConfig::None => None,
            DepthStencilConfig::Standard => {
                let target = match frame.is_output_surface() {
                    true => &mut self.graphics_context.output_depth_stencil,
                    // A more sophicated per-target approach may be wise for depth stencils against render targets, if more than one target is used per frame
                    false => &mut self.graphics_context.depth_stencil,
                };
                let needed_size = frame.get_output_size();
                if let Some(depth_stencil) = target {
                    let current_size = UWimpyPoint::from(depth_stencil.texture.size());
                    if current_size != needed_size {
                        target.take();
                    }
                }
                let depth_stencil = target.get_or_insert_with(||{
                    let device = self.graphics_context.graphics_provider.get_device();
                    DepthStencil::create(device,needed_size)
                });
                Some(RenderPassDepthStencilAttachment {
                    view: &depth_stencil.view,
                    depth_ops: Some(Operations {
                        // '1.0' is the far plane when rendering 'standard' z; i.e., near = '0.0', far = '1.0' [CompareFunction::Less].
                        // Change to '0.0' if using 'reverse' z;               i.e., near = '1.0', far = '0.0' [CompareFunction::Greater]
                        // The clip space in the camera projecton much match this convention as well
                        // (Keep in mind, z world space is not equal to z clip space because we use Z up world space and WGPU uses Y up)
                        load: LoadOp::Clear(1.0),
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                })
            }
        };

        let mut render_pass = self.encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: match frame.get_clear_color() {
                        Some(color) => LoadOp::Clear(color),
                        None => LoadOp::Load,
                    },
                    store: StoreOp::Store,
                },
            })],
            multiview_mask: None,
            depth_stencil_attachment,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        let frame_size = WimpyVec::from(frame.get_input_size());
        render_pass.set_viewport(0.0,0.0,frame_size.x,frame_size.y,0.0,1.0);

        let ortho_uniform = self.graphics_context.pipelines.core.create_uniform_ortho(frame.size());

        Ok(RenderPassBuilder {
            render_pass,
            context: self.graphics_context,
            frame,
            variant_key: pipeline_variant,
            ortho_uniform,
        })
    }

    pub fn create_render_pass<'a,TRenderTarget>(&'a mut self,frame: &'a TRenderTarget) -> Result<RenderPassBuilder<'a,TRenderTarget>,GPUTextureCacheError>
    where
        TRenderTarget: RenderTarget
    {
        self.create_render_pass_internal(frame,DepthStencilConfig::None)
    }

    pub fn create_render_pass_with_depth_stencil<'a,TRenderTarget>(&'a mut self,frame: &'a TRenderTarget) -> Result<RenderPassBuilder<'a,TRenderTarget>,GPUTextureCacheError>
    where
        TRenderTarget: RenderTarget
    {
        self.create_render_pass_internal(frame,DepthStencilConfig::Standard)
    }
}

impl OutputBuilderContext<'_> {
    pub fn present_output_surface(self) {
        let graphics_context = self.builder.graphics_context;

        let queue = graphics_context.graphics_provider.get_queue();
        graphics_context.pipelines.flush(queue);
        queue.submit(std::iter::once(self.builder.encoder.finish()));

        let texture_key = self.frame.get_key();
        if let Err(error) = graphics_context.texture_manager.gpu_cache.remove(texture_key) {
            log::warn!("Output frame was not present in the frame cache: {:?}",error);
        };
        self.builder.output_surface.present();
    }
}
