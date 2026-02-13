mod shader_definitions;
use cgmath::SquareMatrix;
pub use shader_definitions::*;

mod creation;

use crate::app::wam::ModelData;

use super::*;

pub struct Pipeline3D {
    pipeline: RenderPipeline,
    vertex_instance_buffer: DoubleBuffer<ModelInstance>,
}

struct TextureDrawData {
    diffuse: Option<TextureFrame>,
    lightmap: Option<TextureFrame>,
    diffuse_filter: FilterMode,
    strategy: TextureStrategy,
}

impl Pipeline3D {
    pub const VERTEX_BUFFER_INDEX: u32 = 0;
    pub const INSTANCE_BUFFER_INDEX: u32 = 1;
}

impl PipelineController for Pipeline3D {
    fn write_dynamic_buffers(&mut self,queue: &Queue) {
        self.vertex_instance_buffer.write_out(queue);
    }
    fn reset_pipeline_state(&mut self) {
        self.vertex_instance_buffer.reset();
    }
}

pub struct FrameRenderPass3D<'gc,TFrame> {
    context: RenderPassContext<'gc>,
    render_pass: RenderPass<'gc>,
    frame: TFrame,
    has_transform_bind: bool
}

impl<'gc,TFrame> FrameRenderPass<'gc,TFrame> for FrameRenderPass3D<'gc,TFrame>
where 
    TFrame: MutableFrame
{
    fn create(
        frame: TFrame,
        mut render_pass: RenderPass<'gc>,
        context: RenderPassContext<'gc>
    ) -> Self {
        let pipeline_3d = context.get_3d_pipeline();
        render_pass.set_pipeline(&pipeline_3d.pipeline);

        render_pass.set_index_buffer(
            context.model_cache.get_index_buffer_slice(),
            wgpu::IndexFormat::Uint32
        );

        render_pass.set_vertex_buffer(
            Pipeline3D::VERTEX_BUFFER_INDEX,
            context.model_cache.get_vertex_buffer_slice()
        );

        render_pass.set_vertex_buffer(
            Pipeline3D::INSTANCE_BUFFER_INDEX,
            pipeline_3d.vertex_instance_buffer.get_output_buffer().slice(..)
        );

        return Self {
            context,
            render_pass,
            frame,
            has_transform_bind: false,
        }
    }

    fn finish(
        self
    ) -> TFrame {
        return self.frame;
    }
}

#[derive(Copy,Clone)]
pub struct DrawData3D {
    pub transform: Matrix4<f32>,
    pub diffuse_color: WimpyColor,
    pub lightmap_color: WimpyColor,
}

impl Default for DrawData3D {
    fn default() -> Self {
        Self {
            transform: Matrix4::identity(),
            diffuse_color: WimpyColor::WHITE,
            lightmap_color: WimpyColor::WHITE,
        }
    }
}

#[derive(Copy,Clone)]
pub enum TextureStrategy {
    Standard,
    NoLightmap,
    LightmapToDiffuse,
}

impl<TFrame> FrameRenderPass3D<'_,TFrame>
where 
    TFrame: MutableFrame
{
    pub fn set_transform(&mut self,transform: TransformUniform) {
        let uniform_buffer_range = self.context.pipelines
            .get_shared_mut()
            .get_uniform_buffer()
            .push(transform);

        let dynamic_offset = uniform_buffer_range.start * UNIFORM_BUFFER_ALIGNMENT;
        self.render_pass.set_bind_group(
            UNIFORM_BIND_GROUP_INDEX,
            self.context.get_shared().get_uniform_bind_group(),
            &[dynamic_offset as u32]
        );
    }

    pub fn draw(
        &mut self,
        model_data: &ModelData,
        diffuse_filter: FilterMode,
        texture_strategy: TextureStrategy,
        draw_data: &[DrawData3D]
    ) {

        let Some(mesh_reference) = model_data.render else {
            log::warn!("Model data's 'render' value is 'None'. Is this intentional?");
            return;
        };

        if !self.has_transform_bind {
            self.set_transform(TransformUniform::default());
        }

        self.set_mesh_textures(&TextureDrawData {
            diffuse: model_data.diffuse,
            lightmap: model_data.lightmap,
            diffuse_filter,
            strategy: texture_strategy
        });
        todo!();

    }

    fn set_mesh_textures(&mut self,texture_data: &TextureDrawData) {

        let m = &self.context.textures.missing;
        let w = &self.context.textures.opaque_white;

        let Some((diffuse,lightmap)) = self.get_samplers((
            texture_data.diffuse_filter,
            FilterMode::Linear
        ),match (
            &texture_data.diffuse,
            &texture_data.lightmap,
            &texture_data.strategy
        ) {
            (None, None, _) =>                                          (m, w),

            (None, Some(l),     TextureStrategy::Standard) =>           (m, l),
            (Some(d), None,     TextureStrategy::Standard) =>           (d, w),
            (Some(d), Some(l),  TextureStrategy::Standard) =>           (d, l),

            (Some(d), _,        TextureStrategy::NoLightmap) =>         (d, w),
            (None, _,           TextureStrategy::NoLightmap) =>         (m, w),

            (_, Some(l),        TextureStrategy::LightmapToDiffuse) =>  (l, w),
            (_, None,           TextureStrategy::LightmapToDiffuse) =>  (m, w),
        }) else {
            return;
        };
        
        // self.render_pass.set_bind_group(
        //     TEXTURE_BIND_GROUP_INDEX,
        //     diffuse
        //     , offsets);
    }

    fn get_sampler(&self,texture: &TextureFrame,filter_mode: FilterMode) -> Option<&BindGroup> {
        let Ok(texture_container) = self.context.frame_cache.get(texture.get_cache_reference()) else {
            return None;
        };
        return texture_container.get_bind_group(filter_mode,AddressMode::ClampToEdge);
    }

    fn get_samplers(&self,filter_modes: (FilterMode,FilterMode),textures: (&TextureFrame,&TextureFrame)) -> Option<(&BindGroup,&BindGroup)> {
        return match (
            self.get_sampler(textures.0,filter_modes.0),
            self.get_sampler(textures.1,filter_modes.1)
        ) {
            (Some(s1), Some(s2)) => Some((s1,s2)),
            _ => {
                log::error!("Could not locate a texture sampler bind group; skipping this draw command.");
                None
            }
        };
    }
}
