const TEXTURE_CACHE_SLOTMAP_DEFAULT_SIZE: usize =       32;
const UPDATE_OPERATIONS_BUFFER_DEFAULT_SIZE: usize =    16;
const ATLAS_START_QUANTITY: usize =                     2;

use slotmap::SlotMap;

use wgpu::*;
use super::*;
use crate::{UWimpyPoint, WimpyPointRect, app::{WimpyIO, wam::HardAsset}};

use crate::app::graphics::GraphicsProvider;

#[derive(Default,Clone,Copy)]
pub enum StreamingPolicy {
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
pub enum StreamingHint {
    /// No particular stream policy tuning
    #[default]
    None,
    /// Adjusts the streaming policy to optimize for atlas usage
    Atlas,
    /// Tells the streaming policy this texture should always behave as `StaticGPU`
    Static,
}

pub struct TextureCopyParameters {
    source:             WimpyTextureKey,
    destination:        GPUTextureKey, // may need to be changed to a WimpyTextureKey
    source_origin:      UWimpyPoint,
    source_size:        UWimpyPoint,
    destination_origin: UWimpyPoint
}

pub struct TextureCreationParameters {
    pub identity:       HardAsset,
    /// A verified size or a hint provided by WAM.
    /// 
    /// If provided by WAM, the asset's real size in storage may be in disagreement with the manifest
    pub size_hint:           UWimpyPoint,
    pub policy_hint:    StreamingHint,
    pub slice:          Option<WimpyPointRect>
}

pub struct TextureManager {
    pub gpu_cache:              GPUTextureCache,
    pub bind_groups:            BindGroupCache,
    id_generator:               GPUTextureIdentityGenerator,
    states:                     SlotMap<WimpyTextureKey,State>,
    update_queue:     Vec<UpdateOperation>,
    streaming_policy:           StreamingPolicy,
    atlases:                    SlotMap<VirtualTextureAtlasKey,VirtualTextureAtlas>
}

enum UpdateOperation {
    /// Contact operation to stream or refresh texture time to live
    Refresh(WimpyTextureKey),
    CopyTextureToTexture(TextureCopyParameters)
}

struct State {
    wam_identity:       Option<HardAsset>,
    /// The size of the texture according to WAM metadata
    size_hint:          UWimpyPoint,
    /// The size of the texture in a `cache_reference` or `memory_copy`
    size:               Option<UWimpyPoint>,
    local_data:         Option<Vec<u8>>,
    policy_hint:        StreamingHint,
    gpu_texture_key:    Option<GPUTextureKey>,
}

pub struct TextureAtlasConfig {
    /// How big a slot is (e.g., 16 pixels)
    pub slot_size: u32,
    /// How many slots are in the horizontal and vertical dimension (`number of slots` == `slot_length * slot_length`)
    pub slot_length: u32
}

impl TextureManager {
    pub fn new(graphics_provider: &GraphicsProvider,texture_layout: BindGroupLayout,streaming_policy: StreamingPolicy) -> Self {
        Self {
            gpu_cache:              GPUTextureCache::new(),
            states:                 SlotMap::with_capacity_and_key(TEXTURE_CACHE_SLOTMAP_DEFAULT_SIZE),
            update_queue:           Vec::with_capacity(UPDATE_OPERATIONS_BUFFER_DEFAULT_SIZE),
            id_generator:           Default::default(),
            bind_groups:            BindGroupCache::create(graphics_provider.get_device(),texture_layout),
            atlases:                SlotMap::with_capacity_and_key(ATLAS_START_QUANTITY),
            streaming_policy,
        }
    }

    pub fn create_key_for_asset(&mut self,parameters: TextureCreationParameters) -> WimpyTexture {
        let texture_state = State {
            policy_hint:        parameters.policy_hint,
            wam_identity:       Some(parameters.identity),
            size_hint:          parameters.size_hint,
            size:               None,
            local_data:         None,
            gpu_texture_key:    None,
        };
        let key = self.states.insert(texture_state);
        WimpyTexture {
            key,
            size: parameters.size_hint,
            slice: parameters.slice
        }
    }

    pub fn create_static_gpu_texture(&mut self,texture: GPUTexture) -> WimpyTexture {
        let size = texture.size();
        let cache_reference = self.gpu_cache.insert_keyless(texture);
        let texture_state = State {
            wam_identity:       None,
            size_hint:          size,
            size:               Some(size),
            local_data:         None,
            policy_hint:        StreamingHint::Static,
            gpu_texture_key:    Some(cache_reference),
        };
        let key = self.states.insert(texture_state);
        WimpyTexture {
            key,
            size,
            slice: None,
        }
    }

    pub fn copy_texture_to_texture(&mut self,parameters: TextureCopyParameters) {
        self.update_queue.push(UpdateOperation::CopyTextureToTexture(parameters));
    }

    /// Refreshes time to live or restreams the texture to a gpu container
    pub fn touch(&mut self,texture: &WimpyTexture) {
        self.update_queue.push(UpdateOperation::Refresh(texture.key));
    }

    // pub fn get_gpu_texture(&mut self,key: GPUTextureKey) -> &GPUTexture {
    //     match self.gpu_textures.get(key) {
    //         Ok(_) => todo!(),
    //         Err(_) => todo!(),
    //     }
    // }

    // pub fn get_gpu_texture_key(&mut self,source: &impl GPUTextureResolver) -> &GPUTexture {
    //     todo!();
    // }

    pub fn has_updates(&self) -> bool {
        self.update_queue.len() > 0
    }

    pub async fn update<IO: WimpyIO>(&mut self) {
        for update in self.update_queue.drain(..) {
            todo!();
        }
    }

    pub fn create_atlas(
        &mut self,
        graphics_provider: &GraphicsProvider,
        config: &TextureAtlasConfig
    ) -> VirtualTextureAtlasKey {
        //todo: validate size with graphics provider
        todo!()
    }

    pub fn get_atlas(&mut self,key: VirtualTextureAtlasKey) -> Result<&mut VirtualTextureAtlasKey,()> {
        todo!()
    }

    pub fn delete_atlas(&mut self,key: VirtualTextureAtlasKey) -> Result<(),()> {
        todo!()
    }

    // pub fn from_image_unchecked(
    //     &mut self,
    //     graphics_provider: &GraphicsProvider,
    //     size: UWimpyPoint,
    //     data: &[u8]
    // ) -> GPUTexture {
    //     let gpu_texture = GPUTexture::new(graphics_provider,GPUTextureConfig {
    //         size,
    //         identity: self.id_generator.next(),
    //         with_queue_data: true,
    //         render_target: false,
    //     });
    //     graphics_provider.get_queue().write_texture(
    //         TexelCopyTextureInfo {
    //             texture: gpu_texture.get_texture(),
    //             mip_level: 1,
    //             origin: Origin3d::ZERO,
    //             aspect: TextureAspect::All,
    //         },
    //         data,
    //         TexelCopyBufferLayout {
    //             offset: 0,
    //             bytes_per_row: Some(4 * size.x),
    //             rows_per_image: Some(size.y),
    //         },
    //         Extent3d {
    //             width: size.x,
    //             height: size.y,
    //             depth_or_array_layers: 1,
    //         },
    //     );
    //     gpu_texture
    // }

    // pub fn create_gpu_texture(
    //     &mut self,
    //     graphics_provider: &GraphicsProvider,
    //     size: UWimpyPoint,
    //     data: &[u8]
    // ) -> Result<GPUTexture,SizeValidationError> {
    //     graphics_provider.validate_size(size)?;
    //     Ok(self.from_image_unchecked(graphics_provider,size,data))
    // }

    pub fn create_keyless_render_target(
        &mut self,
        graphics_provider: &GraphicsProvider,
        size: UWimpyPoint,
    ) -> GPUTextureKey {
        let texture = GPUTexture::new(graphics_provider,GPUTextureConfig {
            size,
            identity: self.id_generator.next(),
            render_target: true,
            with_queue_data: false,
        });

        self.gpu_cache.insert_keyless(texture)
    }

    pub fn borrow_render_target(
        &mut self,
        graphics_provider: &GraphicsProvider,
        size: UWimpyPoint,
        key: u32,
    ) -> GPUTextureKey {
        match self.gpu_cache.start_lease(key) {
            Ok(value) => value, 
            Err(error) => {
                log::warn!("Graphics context creating a new temp frame. Reason: {:?}",error);
                self.gpu_cache.insert_with_lease(key,GPUTexture::new(graphics_provider,GPUTextureConfig {
                    size,
                    identity: self.id_generator.next(),
                    render_target: true,
                    with_queue_data: false,
                }))
            },
        }
    }

    pub fn ensure_cached_render_target(
        &mut self,
        graphics_provider: &GraphicsProvider,
        size: UWimpyPoint,
        key: u32,
    ) {
        if self.gpu_cache.has_available_items(key) {
            return;
        }
        self.gpu_cache.insert(key,GPUTexture::new(graphics_provider,GPUTextureConfig {
            size,
            identity: self.id_generator.next(),
            render_target: true,
            with_queue_data: false
        }));
    }

    pub fn bind_output_surface(
        &mut self,
        surface: &SurfaceTexture,
        texture_view_format: TextureFormat,
        size: UWimpyPoint // Externally validated (in graphics context)
    ) -> GPUTextureKey {
        let view = surface.texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Output Surface Texture View"),
            format: Some(texture_view_format),
            ..Default::default()
        });
        let texture = GPUTexture {
            identity: GPUTextureIdentity::Anonymous,
            input_size: size,
            view
        };
        self.gpu_cache.insert_keyless(texture)
    }
}

impl CacheResolver for WimpyTexture {
    fn get_cache_entry<'a>(&self,texture_manager: &'a mut TextureManager) -> CacheEntry<'a> {
        todo!()
    }
}

impl CacheResolver for WimpyTextureKey {
    fn get_cache_entry<'a>(&self,texture_manager: &'a mut TextureManager) -> CacheEntry<'a> {
        todo!()
    }
}

impl CacheResolver for render_targets::Output {
    fn get_cache_entry<'a>(&self,texture_manager: &'a mut TextureManager) -> CacheEntry<'a> {
        todo!()
    }
}

impl CacheResolver for render_targets::Temp {
    fn get_cache_entry<'a>(&self,texture_manager: &'a mut TextureManager) -> CacheEntry<'a> {
        todo!()
    }
}

impl CacheResolver for render_targets::LongLife {
    fn get_cache_entry<'a>(&self,texture_manager: &'a mut TextureManager) -> CacheEntry<'a> {
        todo!()
    }
}
