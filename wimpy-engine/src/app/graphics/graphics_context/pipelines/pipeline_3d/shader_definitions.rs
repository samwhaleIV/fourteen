use super::*;

#[repr(C)]
#[derive(Copy,Clone,Debug,Default,Pod,Zeroable)]
pub struct ModelVertex {
    pub diffuse_uv: [f32;2],
    pub lightmap_uv: [f32;2],
    pub position: [f32;3],
}

#[repr(C)]
#[derive(Copy,Clone,Debug,Default,Pod,Zeroable)]
pub struct ModelInstance {
    pub transform_0: [f32;4],
    pub transform_1: [f32;4],
    pub transform_2: [f32;4],
    pub transform_3: [f32;4],
    pub diffuse_color: [u8;4],
    pub lightmap_color: [u8;4]
}

#[non_exhaustive]
struct ATTR;

impl ATTR {
    pub const DIFFUSE_UV: u32 = 0;
    pub const LIGHTMAP_UV: u32 = 1;
    pub const POSITION: u32 = 2;
    pub const TRANSFORM_0: u32 = 3;
    pub const TRANSFORM_1: u32 = 4;
    pub const TRANSFORM_2: u32 = 5;
    pub const TRANSFORM_3: u32 = 6;
    pub const DIFFUSE_COLOR: u32 = 7;
    pub const LIGHTMAP_COLOR: u32 = 8;
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
    const ATTRS: [wgpu::VertexAttribute;6] = wgpu::vertex_attr_array![
        ATTR::TRANSFORM_0 => Float32x4,
        ATTR::TRANSFORM_1 => Float32x4,
        ATTR::TRANSFORM_2 => Float32x4,
        ATTR::TRANSFORM_3 => Float32x4,
        ATTR::DIFFUSE_COLOR => Unorm8x4,
        ATTR::LIGHTMAP_COLOR => Unorm8x4,
    ];

    pub fn get_buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        return wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRS,
        }
    }
}
