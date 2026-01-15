use std::marker::PhantomData;

use crate::{
    shared::{
        CacheArena, CacheArenaConfig, CacheArenaError
    },
    wgpu::{GraphicsContextConfig, texture_container::TextureContainer}
};

struct FrameCacheConfig<TConfig> { phantom_data: PhantomData<TConfig> }

impl<TConfig> CacheArenaConfig for FrameCacheConfig<TConfig> where TConfig: GraphicsContextConfig {
    //May need to optimize these. For example, double the size of entries to reduce arena insertion pressure.
    const ENTRIES: usize = TConfig::INSTANCE_CAPACITY;
    const POOL_COUNT: usize = TConfig::CACHE_SIZES.len();
    const POOL_SIZE: usize = TConfig::CACHE_INSTANCES;
    const LEASES: usize = Self::ENTRIES;
}

pub type FrameCache<TConfig> = CacheArena<(u32,u32),FrameCacheReference,TextureContainer,FrameCacheConfig<TConfig>>;
pub type FrameCacheError = CacheArenaError<(u32,u32),FrameCacheReference>;

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
