pub mod pipeline_2d;
pub mod pipeline_3d;
pub mod text_pipeline;
pub mod lines_pipeline;
pub mod core;

impl crate::app::graphics::RenderPassContext<'_> {
    pub fn get_shared(&self) -> &core::SharedPipeline {
        self.pipelines.get_shared()
    }

    pub fn get_shared_mut(&mut self) -> &mut core::SharedPipeline {
        self.pipelines.get_shared_mut()
    }

    pub fn get_3d_pipeline(&self) -> &pipeline_3d::Pipeline3D {
        &self.pipelines.get_unique().pipeline_3d
    }

    pub fn get_3d_pipeline_mut(&mut self) -> &mut pipeline_3d::Pipeline3D {
        &mut self.pipelines.get_unique_mut().pipeline_3d
    }

    pub fn get_2d_pipeline(&self) -> &pipeline_2d::Pipeline2D {
        &self.pipelines.get_unique().pipeline_2d
    }

    pub fn get_2d_pipeline_mut(&mut self) -> &mut pipeline_2d::Pipeline2D {
        &mut self.pipelines.get_unique_mut().pipeline_2d
    }

    pub fn get_text_pipeline(&self) -> &text_pipeline::TextPipeline {
        &self.pipelines.get_unique().text_pipeline
    }

    pub fn get_text_pipeline_mut(&mut self) -> &mut text_pipeline::TextPipeline {
        &mut self.pipelines.get_unique_mut().text_pipeline
    }

    pub fn get_line_pipeline(&self) -> &lines_pipeline::LinesPipeline {
        &self.pipelines.get_unique().lines_pipeline
    }

    pub fn get_line_pipeline_mut(&mut self) -> &mut lines_pipeline::LinesPipeline {
        &mut self.pipelines.get_unique_mut().lines_pipeline
    }

    pub fn set_texture_bind_group(
        &mut self,
        index: u32,
        render_pass: &mut wgpu::RenderPass,
        bind_group_identity: &crate::app::graphics::BindGroupCacheIdentity
    ) {
        let bind_group = self.bind_groups.get(self.graphics_provider.get_device(),bind_group_identity);
        render_pass.set_bind_group(index,bind_group,&[]);
    }
}
