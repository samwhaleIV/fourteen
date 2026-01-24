use bytemuck::{
    Pod,
    Zeroable
};

use crate::wgpu::DrawData;

#[repr(C)]
#[derive(Copy,Clone,Debug,Default,Pod,Zeroable)]
pub struct Vertex {
    pub position: [f32;2],
    //_padding: [f32;2]
}

#[repr(C)]
#[derive(Copy,Clone,Debug,Default,Pod,Zeroable)]
pub struct QuadInstance { //Aligned to 64
    pub position: [f32;2],
    pub size: [f32;2],
    pub uv_position: [f32;2],
    pub uv_size: [f32;2],
    pub color: [f32;4],
    pub rotation: f32,
    pub _padding: [f32;3]
}

#[repr(C)]
#[derive(Debug,Copy,Clone,Pod,Zeroable)]
pub struct CameraUniform {
    pub view_projection: [[f32;4];4]
}

#[non_exhaustive]
struct ATTR;

impl ATTR {
    pub const VERTEX_POSITION: u32 = 0;
    pub const INSTANCE_POSITION: u32 = 1;
    pub const SIZE: u32 = 2;
    pub const UV_POS: u32 = 3;
    pub const UV_SIZE: u32 = 4;
    pub const COLOR: u32 = 5;
    pub const ROTATION: u32 = 6;
}

impl Vertex {
    const ATTRS: [wgpu::VertexAttribute;1] = wgpu::vertex_attr_array![
        ATTR::VERTEX_POSITION => Float32x2,
    ];

    pub fn get_buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        return wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRS,
        }
    }
}

impl QuadInstance {
    const ATTRS: [wgpu::VertexAttribute;6] = wgpu::vertex_attr_array![
        ATTR::INSTANCE_POSITION => Float32x2,
        ATTR::SIZE => Float32x2,
        ATTR::UV_POS => Float32x2,
        ATTR::UV_SIZE => Float32x2,
        ATTR::COLOR => Float32x4,
        ATTR::ROTATION => Float32,
    ];

    pub fn get_buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        return wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRS,
        }
    }
}

impl<'a> From<&'a DrawData> for QuadInstance {
    fn from(value: &'a DrawData) -> Self {
        let area = value.destination.to_center_encoded();
        return QuadInstance {
            position: [
                area.x,
                area.y,
            ],
            size: [
                area.width,
                area.height,
            ],
            uv_position: [
                value.source.x,
                value.source.y,
            ],
            uv_size: [
                value.source.width,
                value.source.height,
            ],
            color: value.color.to_float_array(),
            rotation: value.rotation,
            _padding: [0.0,0.0,0.0],
        }
    }
}

impl From<DrawData> for QuadInstance {
    fn from(value: DrawData) -> Self {
        let area = value.destination.to_center_encoded();
        return QuadInstance {
            position: [
                area.x,
                area.y,
            ],
            size: [
                area.width,
                area.height,
            ],
            uv_position: [
                value.source.x,
                value.source.y,
            ],
            uv_size: [
                value.source.width,
                value.source.height,
            ],
            color: value.color.to_float_array(),
            rotation: value.rotation,
            _padding: [0.0,0.0,0.0],
        }
    }
}
