use std::marker::PhantomData;

use crate::{
    shared::{
        CacheArena,
        CacheArenaConfig,
        CacheArenaError
    },
    wgpu::texture_container::TextureContainer
};

pub struct FrameCacheConfig<TConfig> { phantom_data: PhantomData<TConfig> }

impl<TConfig> CacheArenaConfig for FrameCacheConfig<TConfig> {
    const ENTRIES: usize = 256;
    const LEASES: usize = 256;
    const POOL_COUNT: usize = 16;
    const POOL_SIZE: usize = 16;
}

pub type FrameCache<TConfig> = CacheArena<u32,FrameCacheReference,TextureContainer,FrameCacheConfig<TConfig>>;
pub type FrameCacheError = CacheArenaError<u32,FrameCacheReference>;

slotmap::new_key_type! {
    pub struct FrameCacheReference;
}

pub trait FrameCacheLookup {
    fn get_texture_container(&self,reference: FrameCacheReference) ->  Result<&TextureContainer,FrameCacheError>;
}

impl<TConfig> FrameCacheLookup for FrameCache<TConfig> {
    fn get_texture_container(&self,reference: FrameCacheReference) -> Result<&TextureContainer,FrameCacheError> {
        return self.get(reference);
    }
}
