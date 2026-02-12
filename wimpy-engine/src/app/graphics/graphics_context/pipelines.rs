use super::prelude::*;

mod draw_data_set_buffers;
use draw_data_set_buffers::*;

pub mod pipeline_shared;
pub mod pipeline_core;
pub mod pipeline_2d;
pub mod pipeline_3d;

pub use pipeline_shared::*;
pub use pipeline_core::*;
pub use pipeline_2d::*;
pub use pipeline_3d::*;

impl RenderPassView<'_> {
    pub fn get_3d_pipeline(&self) -> &Pipeline3D {
        return &self.render_pipelines.get_unique().pipeline_3d;
    }
    pub fn get_3d_pipeline_mut(&mut self) -> &mut Pipeline3D {
        return &mut self.render_pipelines.get_unique_mut().pipeline_3d;
    }

    pub fn get_2d_pipeline(&self) -> &Pipeline2D {
        return &self.render_pipelines.get_unique().pipeline_2d;
    }
    pub fn get_2d_pipeline_mut(&mut self) -> &mut Pipeline2D {
        return &mut self.render_pipelines.get_unique_mut().pipeline_2d;
    }
}
