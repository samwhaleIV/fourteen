mod shader_definitions;
pub use shader_definitions::*;

mod creation;
use super::*;

pub struct Pipeline3D {
    pipeline: RenderPipeline,
    instance_buffer: DoubleBuffer<ModelInstance>,
}

impl Pipeline3D {
    pub const VERTEX_BUFFER_INDEX: u32 = 0;
    pub const INSTANCE_BUFFER_INDEX: u32 = 1;
}

impl PipelineController for Pipeline3D {
    fn write_dynamic_buffers(&mut self,queue: &Queue) {
        self.instance_buffer.write_out(queue);
    }
    fn reset_pipeline_state(&mut self) {
        self.instance_buffer.reset();
    }
}

pub struct FrameRenderPass3D<TFrame> {
    frame: TFrame,
    transform: MatrixTransformUniform,
}

impl<TFrame> FrameRenderPass<TFrame> for FrameRenderPass3D<TFrame>
where 
    TFrame: MutableFrame
{
    fn create(frame: TFrame,render_pass_view: &mut RenderPassView) -> Self {
        todo!()
    }

    fn begin_render_pass(
        self,
        render_pass: &mut RenderPass,
        render_pass_view: &mut RenderPassView
    ) -> TFrame {
        let pipeline_3d = render_pass_view.get_3d_pipeline();

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
            pipeline_3d.instance_buffer.get_output_buffer().slice(..)
        );

        let shared_pipeline = render_pass_view.get_shared_pipeline_mut();
        let uniform_buffer_range = shared_pipeline.get_uniform_buffer().push(self.transform);
        let dynamic_offset = uniform_buffer_range.start * UNIFORM_BUFFER_ALIGNMENT;

        render_pass.set_bind_group(
            UNIFORM_BIND_GROUP_INDEX,
            shared_pipeline.get_uniform_bind_group(),
            &[dynamic_offset as u32]
        );

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

impl<TFrame> FrameRenderPass3D<TFrame>
where 
    TFrame: MutableFrame
{
    fn create(frame: TFrame,transform: MatrixTransformUniform) -> Self {
        return Self {
            frame,
            transform
        }
    }

    pub fn draw(&mut self,model_reference: ModelCacheReference,draw_data: DrawData3D) {
        //todo
    }
}

impl<'a> From<&'a DrawData3D> for ModelInstance {
    fn from(value: &'a DrawData3D) -> Self {
        return ModelInstance {
            transform_0: value.transform.x.into(),
            transform_1: value.transform.y.into(),
            transform_2: value.transform.z.into(),
            transform_3: value.transform.w.into(),
            diffuse_color: value.diffuse_color.decompose(),
            lightmap_color: value.lightmap_color.decompose(),
        }
    }
}

impl From<DrawData3D> for ModelInstance {
    fn from(value: DrawData3D) -> Self {
        ModelInstance::from(&value)
    }
}
