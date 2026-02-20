use super::prelude::*;

mod pipeline_creator;
mod pipeline_shared;
mod pipeline_core;
mod pipeline_2d;
mod pipeline_3d;
mod text_pipeline;

pub use pipeline_creator::*;
pub use pipeline_shared::*;
pub use pipeline_core::*;
pub use pipeline_2d::*;
pub use pipeline_3d::*;
pub use text_pipeline::*;

impl RenderPassContext<'_> {
    pub fn get_shared(&self) -> &SharedPipeline {
        return self.pipelines.get_shared();
    }
    pub fn get_shared_mut(&mut self) -> &mut SharedPipeline {
        return self.pipelines.get_shared_mut();
    }
    pub fn get_3d_pipeline(&self) -> &Pipeline3D {
        return &self.pipelines.get_unique().pipeline_3d;
    }
    pub fn get_3d_pipeline_mut(&mut self) -> &mut Pipeline3D {
        return &mut self.pipelines.get_unique_mut().pipeline_3d;
    }
    pub fn get_2d_pipeline(&self) -> &Pipeline2D {
        return &self.pipelines.get_unique().pipeline_2d;
    }
    pub fn get_2d_pipeline_mut(&mut self) -> &mut Pipeline2D {
        return &mut self.pipelines.get_unique_mut().pipeline_2d;
    }
    pub fn get_text_pipeline(&self) -> &TextPipeline {
        return &self.pipelines.get_unique().text_pipeline;
    }
    pub fn get_text_pipeline_mut(&mut self) -> &mut TextPipeline {
        return &mut self.pipelines.get_unique_mut().text_pipeline;
    }
    pub fn set_texture_bind_group(&mut self,render_pass: &mut RenderPass,bind_group_identity: &BindGroupCacheIdentity) {
        let bind_group = self.bind_groups.get(self.graphics_provider.get_device(),bind_group_identity);
        render_pass.set_bind_group(TEXTURE_BIND_GROUP_INDEX,bind_group,&[]);
    }
}
