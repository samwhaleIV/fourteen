mod constants;

mod engine_textures;
use engine_textures::EngineTextures;

pub mod textures;

mod mesh_cache;
pub use mesh_cache::*;

mod double_buffer;
pub use double_buffer::DoubleBuffer;

pub mod pipelines;

mod graphics_provider;
mod graphics_context;

pub use graphics_provider::*;
pub use graphics_context::*;

#[derive(Debug)]
pub enum SizeValidationError {
    TooSmall {
        value: u32,
        limit: u32
    },
    TooBig {
        value: u32,
        limit: u32,
    }
}

pub trait GraphicsConfig {
    // These are in byte count
    const UNIFORM_BUFFER_SIZE: usize;
    const INSTANCE_BUFFER_SIZE_2D: usize;
    const MESH_CACHE_VERTEX_BUFFER_SIZE: usize;
    const MESH_CACHE_INDEX_BUFFER_SIZE: usize;
    const INSTANCE_BUFFER_SIZE_3D: usize;
    const TEXT_PIPELINE_BUFFER_SIZE: usize;
    const LINE_BUFFER_SIZE: usize;
}
