use super::prelude::*;

pub mod pipeline_shared;
pub mod pipeline_core;
pub mod pipeline_2d;
pub mod pipeline_3d;

pub use pipeline_shared::*;
pub use pipeline_core::*;
pub use pipeline_2d::*;
pub use pipeline_3d::*;

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
}
