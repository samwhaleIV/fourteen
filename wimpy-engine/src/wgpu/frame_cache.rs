use crate::{
    shared::{
        CacheArena,
        CacheArenaConfig,
        CacheArenaError
    },
    wgpu::texture_container::TextureContainer
};

pub struct FrameCacheConfig;

impl CacheArenaConfig for FrameCacheConfig {
    const ENTRIES: usize = 256;
    const LEASES: usize = 256;
    const POOL_COUNT: usize = 16;
    const POOL_SIZE: usize = 16;
}

pub type FrameCache = CacheArena<u32,FrameCacheReference,TextureContainer,FrameCacheConfig>;
pub type FrameCacheError = CacheArenaError<u32,FrameCacheReference>;

slotmap::new_key_type! {
    pub struct FrameCacheReference;
}

pub trait FrameCacheLookup {
    fn get_texture_container(&self,reference: FrameCacheReference) ->  Result<&TextureContainer,FrameCacheError>;
}

impl FrameCacheLookup for FrameCache {
    fn get_texture_container(&self,reference: FrameCacheReference) -> Result<&TextureContainer,FrameCacheError> {
        return self.get(reference);
    }
}
