use bytemuck::*;

use wgpu::{
    Buffer,
    RenderPipeline
};

use crate::wgpu::{
    DrawData3D, GraphicsProvider, pipelines::{
        CameraUniform,
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
        uniform: CameraUniform
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
        uniform: CameraUniform
    ) {
        todo!()
    }
}


#[repr(C)]
#[derive(Copy,Clone,Debug,Default,Pod,Zeroable)]
pub struct ModelVertex {
    pub diffuse_uv: [f32;2],
    pub lightmap_uv: [f32;2],
    pub position: [f32;3],
    _padding: [f32;1],
}

#[repr(C)]
#[derive(Copy,Clone,Debug,Default,Pod,Zeroable)]
pub struct ModelInstance {
    pub diffuse_color: [f32;4],
    pub lightmap_color: [f32;4]
}

#[non_exhaustive]
struct ATTR;

impl ATTR {
    pub const DIFFUSE_UV: u32 = 0;
    pub const LIGHTMAP_UV: u32 = 1;
    pub const POSITION: u32 = 2;
    pub const DIFFUSE_COLOR: u32 = 3;
    pub const LIGHTMAP_COLOR: u32 = 4;
}

impl ModelVertex {
    const ATTRS: [wgpu::VertexAttribute;3] = wgpu::vertex_attr_array![
        ATTR::DIFFUSE_UV => Float32x2,
        ATTR::LIGHTMAP_UV => Float32x2,
        ATTR::POSITION => Float32x3
    ];

    pub fn get_buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        return wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRS,
        }
    }
}

impl ModelInstance {
    const ATTRS: [wgpu::VertexAttribute;2] = wgpu::vertex_attr_array![
        ATTR::DIFFUSE_COLOR => Float32x4,
        ATTR::LIGHTMAP_COLOR => Float32x4,
    ];

    pub fn get_buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        return wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRS,
        }
    }
}

impl<'a> From<&'a DrawData3D> for ModelInstance {
    fn from(value: &'a DrawData3D) -> Self {
        return ModelInstance {
            diffuse_color: value.diffuse_color.to_float_array(),
            lightmap_color: value.lightmap_color.to_float_array(),
        }
    }
}

impl From<DrawData3D> for ModelInstance {
    fn from(value: DrawData3D) -> Self {
        ModelInstance::from(&value)
    }
}
