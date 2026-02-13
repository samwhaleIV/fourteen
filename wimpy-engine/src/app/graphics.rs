mod texture_container;
mod graphics_provider;
mod constants;
mod double_buffer;
mod graphics_context;
mod frame;
mod frame_cache;
mod model_cache;
mod util;

mod prelude {
    pub use std::marker::PhantomData;
    pub use std::num::NonZero;
    pub use std::ops::Range;
    pub use wgpu::util::{
        DeviceExt,
        BufferInitDescriptor
    };
    pub use cgmath::Matrix4;
    pub use crate::shared::*;
    pub use bytemuck::{
        Pod,
        Zeroable
    };
    pub use wgpu::*;
    pub use super::*;
    pub use constants::*;
    pub use double_buffer::*;
    pub use frame_cache::*;
    pub use model_cache::*;
    pub use texture_container::*;
    pub use super::util::*;
}

pub use texture_container::{
    TextureDataWriteParameters,
    TextureData,
};

pub use graphics_provider::*;
pub use frame::*;
pub use model_cache::*;
pub use graphics_context::*;
