use super::TextureContainer;

use crate::collections::cache_arena::*;
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
