use wgpu::{
    Buffer,
    RenderPipeline
};

use crate::wgpu::{
    GraphicsProvider,
    pipelines::{
        RenderPassController,
        SharedPipelineSet
    }
};

pub struct Pipeline3D {
    pipeline: RenderPipeline,
    vertex_buffer: Buffer,
}

impl Pipeline3D {
    pub fn create<TConfig>(
        graphics_provider: &GraphicsProvider,
        shared_pipeline_set: &SharedPipelineSet
    ) -> Self {
        todo!();
    }
}

impl RenderPassController for Pipeline3D {
    fn begin(
        &mut self,
        render_pass: &mut wgpu::RenderPass,
        shared_pipeline: &mut SharedPipelineSet,
        uniform: crate::wgpu::shader_definitions::CameraUniform
    ) {
        todo!()
    }

    fn write_buffers(&mut self,queue: &wgpu::Queue) {
        todo!()
    }

    fn reset_buffers(&mut self) {
        todo!()
    }
    
    fn select_and_begin(
        render_pass: &mut wgpu::RenderPass,
        render_pipelines: &mut super::RenderPipelines,
        uniform: crate::wgpu::shader_definitions::CameraUniform
    ) {
        todo!()
    }
}
