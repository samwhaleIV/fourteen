mod texture_manager;
pub use texture_manager::*;

mod virtual_texture_atlas;
pub use virtual_texture_atlas::*;

mod gpu_texture;
pub use gpu_texture::*;

mod bind_group_cache;
pub use bind_group_cache::*;

mod render_targets;

pub use render_targets::{
    LongLife as LongLifeRenderTarget,
    Output as OutputRenderTarget,
    Temp as TempRenderTarget,
    RenderTarget,
};

use crate::{UWimpyPoint, WimpyVec, WimpyRect, WimpyPointRect, collections::cache_arena::*};

pub struct GPUTextureCacheConfig;

impl CacheArenaConfig for GPUTextureCacheConfig {
    const ENTRIES: usize =      256;
    const LEASES: usize =       256;
    const POOL_COUNT: usize =   16;
    const POOL_SIZE: usize =    16;
}

pub type GPUTextureCache = CacheArena<u32,GPUTextureKey,GPUTexture,GPUTextureCacheConfig>;
pub type GPUTextureCacheError = CacheArenaError<u32,GPUTextureKey>;

slotmap::new_key_type! {
    pub struct GPUTextureKey;
    pub struct WimpyTextureKey;
    pub struct VirtualTextureAtlasKey;
}

#[derive(PartialEq,Eq,Copy,Clone,Hash)]
pub enum SamplerMode {
    NearestClamp,
    NearestWrap,
    NearestWrapMirror,
    LinearClamp,
    LinearWrap,
    LinearWrapMirror
}

#[derive(Copy,Clone)]
pub struct WimpyTexture {
    /// Internal cache arena reference
    pub key:    WimpyTextureKey,
    /// The size of the asset as suggested by WAM. Could be wrong is the asset does not match the object from storage.
    pub size:   UWimpyPoint,
    /// If provided by WAM, this is the suggested sub-area of the texture to use, such as when WAM generates an offline atlas.
    pub slice:  Option<WimpyPointRect>
}

impl WimpyTexture {
    pub fn width(&self) -> u32 {
        self.size.x
    }
    pub fn height(&self) -> u32 {
        self.size.y
    }
    pub fn aspect_ratio(&self) -> f32 {
        self.size.x as f32 / self.size.y as f32
    }
}

#[derive(Copy,Clone)]
pub struct FilteredSize {
    pub input:  UWimpyPoint,
    pub output: UWimpyPoint,
}

pub trait SizeInfo {
    /// The size of the frame as requested by the user. In the case of an imported texture frame, this is its original size.
    fn get_input_size(&self) -> UWimpyPoint;

    /// The size of the real texture this frame renders to.
    fn get_output_size(&self) -> UWimpyPoint;

    fn get_uv_scale(&self) -> WimpyVec {
        let input = self.get_input_size();
        let output = self.get_output_size();

        WimpyVec::from(input) / WimpyVec::from(output)
    }

    fn width(&self) -> u32 {
        self.get_input_size().x
    }

    fn height(&self) -> u32 {
        self.get_input_size().y
    }

    fn size(&self) -> UWimpyPoint {
        self.get_input_size()
    }

    fn area(&self) -> WimpyRect {
        WimpyRect {
            position: WimpyVec::ZERO,
            size: self.get_input_size().into()
        }
    }

    fn aspect_ratio(&self) -> f32 {
        let size = self.get_input_size();
        size.x as f32 / size.y as f32
    }
}

pub trait CacheResolver {
    fn get_cache_entry<'a>(&self,texture_manager: &'a mut TextureManager) -> CacheEntry<'a>;
}

pub struct TextureData {
    data: Vec<u8>,
    size: UWimpyPoint
}

pub struct CacheEntry<'a> {
    pub size:   UWimpyPoint,
    pub key:    GPUTextureKey,
    pub value:  &'a GPUTexture
}
