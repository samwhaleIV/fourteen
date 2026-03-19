mod pipeline_2d;
mod pipeline_3d;
mod text_pipeline;
mod lines_pipeline;

pub use pipeline_2d::*;
pub use pipeline_3d::*;
pub use text_pipeline::*;
pub use lines_pipeline::*;

mod core;
pub use core::*;

use glam::Vec3;
use crate::{WimpyRect, WimpyColorLinear, WimpyVec, WimpyNamedColor};

pub trait PipelinePass<'pass,'context> {
    fn create(
        render_pass:        &'pass mut wgpu::RenderPass<'context>,
        context:            &'pass mut super::GraphicsContext,
        variant_key:        PipelineVariantKey,
        uniform_reference:  UniformReference
    ) -> Self;
}
