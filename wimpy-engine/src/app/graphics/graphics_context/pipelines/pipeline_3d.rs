mod shader_definitions;
pub use shader_definitions::*;

mod creation;

use crate::{
    app::wam::ModelData,
    collections::VecPool
};

use super::*;

pub struct Pipeline3D {
    pipeline: RenderPipeline,
    vertex_instance_buffer: DoubleBuffer<ModelInstance>,
    command_buffer_pool: VecPool<Pipeline3DCommand,DEFAULT_COMMAND_BUFFER_SIZE>,
    set_buffers: DrawDataSetBuffers<DrawData3D,DrawDataSetReference3D>,
}

slotmap::new_key_type! {
    pub struct DrawDataSetReference3D;
}

impl SetBuffersSelector<DrawData3D> for DrawDataSetReference3D {
    fn select(pipelines: &mut RenderPipelines) -> &mut DrawDataSetBuffers<DrawData3D,Self> {
        return &mut pipelines.get_unique_mut().pipeline_3d.set_buffers;
    }
}

enum Pipeline3DCommand {
    //If no transform is set by the first draw call, set the identity matrix
    SetTransform(MatrixTransformUniform),
    Draw {
        mesh_reference: RenderBufferReference,
        diffuse: Option<TextureFrame>,
        lightmap: Option<TextureFrame>,
        texture_mode: TextureMode,
        draw_data: DrawData3D
    },
    DrawInstanced {
        mesh_reference: RenderBufferReference,
        diffuse: Option<TextureFrame>,
        lightmap: Option<TextureFrame>,
        texture_mode: TextureMode,
        draw_data: DrawDataSetReference3D
    }
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

pub struct FrameRenderPass3D<TFrame> {
    frame: TFrame,

    command_buffer: Vec<Pipeline3DCommand>
}

impl<TFrame> FrameRenderPass<TFrame> for FrameRenderPass3D<TFrame>
where 
    TFrame: MutableFrame
{
    fn create(frame: TFrame,render_pass_view: &mut RenderPassView) -> Self {
        let command_buffer = render_pass_view.get_3d_pipeline_mut().command_buffer_pool.take_item();
        return Self {
            frame,
            command_buffer,
        }
    }

    fn begin_render_pass(
        self,
        render_pass: &mut RenderPass,
        render_pass_view: &mut RenderPassView
    ) -> TFrame {
        let pipeline_3d = render_pass_view.get_3d_pipeline();
        render_pass.set_pipeline(&pipeline_3d.pipeline);

        render_pass.set_index_buffer(
            render_pass_view.model_cache.get_index_buffer_slice(),
            wgpu::IndexFormat::Uint32
        );

        render_pass.set_vertex_buffer(
            Pipeline3D::VERTEX_BUFFER_INDEX,
            render_pass_view.model_cache.get_vertex_buffer_slice()
        );

        render_pass.set_vertex_buffer(
            Pipeline3D::INSTANCE_BUFFER_INDEX,
            pipeline_3d.vertex_instance_buffer.get_output_buffer().slice(..)
        );

        // CALL THIS IN THE COMMAND PROCESSOR FOR SETTING THE TRANSFORM
        // let shared_pipeline = render_pass_view.get_shared_pipeline_mut();
        // let uniform_buffer_range = shared_pipeline.get_uniform_buffer().push(self.transform);
        // let dynamic_offset = uniform_buffer_range.start * UNIFORM_BUFFER_ALIGNMENT;

        // render_pass.set_bind_group(
        //     UNIFORM_BIND_GROUP_INDEX,
        //     shared_pipeline.get_uniform_bind_group(),
        //     &[dynamic_offset as u32]
        // );

        render_pass_view.get_3d_pipeline_mut().command_buffer_pool.return_item(self.command_buffer);

        self.frame
    }

    fn get_frame(&self) -> &TFrame {
        return &self.frame;
    }
    
    fn get_frame_mut(&mut self) -> &mut TFrame {
        return &mut self.frame;
    }
}

pub struct DrawData3D {
    pub transform: Matrix4<f32>,
    pub diffuse_color: WimpyColor,
    pub lightmap_color: WimpyColor,
}

impl Default for DrawData3D {
    fn default() -> Self {
        Self {
            transform: get_identity_matrix(),
            diffuse_color: WimpyColor::WHITE,
            lightmap_color: WimpyColor::WHITE,
        }
    }
}

pub enum TextureMode {
    Standard,
    NoLightmap,
    LightmapToDiffuse,
}

impl<TFrame> FrameRenderPass3D<TFrame>
where 
    TFrame: MutableFrame
{
    pub fn set_transform(&mut self,transform: MatrixTransformUniform) {
        self.command_buffer.push(Pipeline3DCommand::SetTransform(transform));
    }

    pub fn draw(
        &mut self,
        model_data: &ModelData,
        texture_mode: TextureMode,
        draw_data: DrawData3D
    ) {
        let Some(mesh_reference) = model_data.render else {
            log::warn!("Model data's 'render' value is 'None'. Is this intentional?");
            return;
        };

        self.command_buffer.push(Pipeline3DCommand::Draw {
            mesh_reference,
            diffuse: model_data.diffuse,
            lightmap: model_data.lightmap,
            texture_mode,
            draw_data
        });
    }

    pub fn draw_instances(
        &mut self,
        model_data: &ModelData,
        texture_mode: TextureMode,
        draw_data: DrawDataSetReference3D
    ) {
        let Some(mesh_reference) = model_data.render else {
            log::warn!("Model data's 'render' value is 'None'. Is this intentional?");
            return;
        };

        self.command_buffer.push(Pipeline3DCommand::DrawInstanced {
            mesh_reference,
            diffuse: model_data.diffuse,
            lightmap: model_data.lightmap,
            texture_mode,
            draw_data
        });
    }
}

fn map_textures_by_mode(
    diffuse: Option<TextureFrame>,
    lightmap: Option<TextureFrame>,
    texture_mode: TextureMode,
    textures: &RuntimeTextures
) -> (TextureFrame, TextureFrame) {
    let m = textures.missing;
    let w = textures.opaque_white;
    return match (diffuse,lightmap,texture_mode) {
        (None, None, _) =>                                      (m, w),

        (None, Some(l),     TextureMode::Standard) =>           (m, l),
        (Some(d), None,     TextureMode::Standard) =>           (d, w),
        (Some(d), Some(l),  TextureMode::Standard) =>           (d, l),

        (Some(d), _,        TextureMode::NoLightmap) =>         (d, w),
        (None, _,           TextureMode::NoLightmap) =>         (m, w),

        (_, Some(l),        TextureMode::LightmapToDiffuse) =>  (l, w),
        (_, None,           TextureMode::LightmapToDiffuse) =>  (m, w),
    }
}
