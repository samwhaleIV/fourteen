use slotmap::SlotMap;

use super::TextureContainer;

use crate::{UWimpyPoint, WimpyPointRect, app::{WimpyIO, graphics::TextureFrame, wam::{HardAsset, WimpyTexture}}, collections::cache_arena::*};
pub struct FrameCacheConfig;

impl CacheArenaConfig for FrameCacheConfig {
    const ENTRIES: usize = 256;
    const LEASES: usize = 256;
    const POOL_COUNT: usize = 16;
    const POOL_SIZE: usize = 16;
}

const TEXTURE_CACHE_SLOTMAP_DEFAULT_SIZE: usize = 32;
const UPDATE_OPERATIONS_BUFFER_DEFAULT_SIZE: usize = 16;

pub type FrameCache = CacheArena<u32,FrameCacheReference,TextureContainer,FrameCacheConfig>;
pub type FrameCacheError = CacheArenaError<u32,FrameCacheReference>;

slotmap::new_key_type! {
    pub struct FrameCacheReference;
    pub struct WimpyTextureKey;
}

pub struct LocalTexture {
    data: Vec<u8>,
    size: UWimpyPoint
}

#[derive(Default,Clone,Copy)]
pub enum TextureStreamPolicy {
    /// Can be loaded from hard storage multiple times upon cache's time to live determinations
    /// 
    /// May be dropped from local memory when a GPU resource is created
    #[default]
    Default,

    /// Local memory copy retained even after a GPU resource is created
    /// 
    /// May read from storage multiple times if the GPU resource is lost
    /// 
    /// More useful in the browser enviroment or consoles if the total mass of game assets are on the smaller side
    Retained,

    /// Suggestion to only load from hard storage once
    /// 
    /// GPU only textures cannot be unloaded
    StaticGPU,
}

#[derive(Default,Clone,Copy)]
pub enum TextureStreamingHint {
    /// No particular stream policy tuning
    #[default]
    None,
    /// Adjusts the streaming policy to optimize for atlas usage
    Atlas,
    /// Tells the streaming policy this texture should always behave as `StaticGPU`
    Static,
}

pub struct TextureState {
    asset_identity: Option<HardAsset>,
    /// The size of the texture according to WAM metadata
    size_hint: UWimpyPoint,
    /// The size of the texture in a `cache_reference` or `memory_copy`
    real_size: Option<UWimpyPoint>,
    memory_copy: Option<Vec<u8>>,
    policy_hint: TextureStreamingHint,
    cache_reference: Option<FrameCacheReference>,
}

enum CacheUpdateOperation {
    /// Contact operation to stream or refresh texture time to live
    Refresh(WimpyTextureKey),
    CopyTextureToTexture(TextureCopyParameters)
}

pub struct TextureCache {
    pub frames: FrameCache,
    textures: SlotMap<WimpyTextureKey,TextureState>,
    update_operations: Vec<CacheUpdateOperation>,
    stream_policy: TextureStreamPolicy,
}

pub struct TextureCopyParameters {
    source: WimpyTextureKey,
    destination: FrameCacheReference, // may need to be changed to a WimpyTextureKey
    source_origin: UWimpyPoint,
    source_size: UWimpyPoint,
    destination_origin: UWimpyPoint
}

pub enum TextureCopyError {

}

pub struct TextureKeyCreationParameters {
    pub identity: HardAsset,
    pub size_hint: UWimpyPoint,
    pub policy_hint: TextureStreamingHint,
    pub slice: Option<WimpyPointRect>
}

impl TextureCache {
    pub fn new(stream_policy: TextureStreamPolicy) -> Self {
        Self {
            frames: FrameCache::new(),
            textures: SlotMap::with_capacity_and_key(TEXTURE_CACHE_SLOTMAP_DEFAULT_SIZE),
            update_operations: Vec::with_capacity(UPDATE_OPERATIONS_BUFFER_DEFAULT_SIZE),
            stream_policy,
        }
    }

    pub fn create_key_for_asset(&mut self,parameters: TextureKeyCreationParameters) -> WimpyTexture {
        let texture_state = TextureState {
            policy_hint: parameters.policy_hint,
            asset_identity: Some(parameters.identity),
            size_hint: parameters.size_hint,
            real_size: None,
            memory_copy: None,
            cache_reference: None,
        };
        let key = self.textures.insert(texture_state);
        WimpyTexture {
            key,
            size: parameters.size_hint,
            slice: parameters.slice
        }
    }

    pub fn create_static_gpu_texture(&mut self,texture_container: TextureContainer) -> WimpyTexture {
        let size = texture_container.size();
        let cache_reference = self.frames.insert_keyless(texture_container);
        let texture_state = TextureState {
            asset_identity: None,
            size_hint: size,
            real_size: Some(size),
            memory_copy: None,
            policy_hint: TextureStreamingHint::Static,
            cache_reference: Some(cache_reference),
        };
        let key = self.textures.insert(texture_state);
        WimpyTexture {
            key,
            size,
            slice: None,
        }
    }

    pub fn copy_texture_to_frame(&mut self,parameters: TextureCopyParameters) {
        self.update_operations.push(CacheUpdateOperation::CopyTextureToTexture(parameters));
    }

    /// Refreshes time to live or restreams the texture to a gpu container
    pub fn touch(&mut self,texture: &WimpyTexture) {
        self.update_operations.push(CacheUpdateOperation::Refresh(texture.key));
    }

    pub fn get_missing_texture(&self) -> &WimpyTexture {
        todo!();
    }

    pub fn has_updates(&self) -> bool {
        self.update_operations.len() > 0
    }

    pub async fn update<IO: WimpyIO>(&mut self) {
        for update in self.update_operations.drain(..) {
            todo!();
        }
    }
}
