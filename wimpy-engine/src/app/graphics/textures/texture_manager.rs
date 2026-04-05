const UPDATE_OPERATIONS_BUFFER_DEFAULT_SIZE: usize =    16;
const ATLAS_START_QUANTITY: usize =                     2;

use wgpu::*;
use slotmap::SlotMap;
use std::num::NonZeroU32;

use super::{*,bind_group_cache::{BindGroupCache, BindGroupChannelSet, BindGroupChannel}};
use crate::{UWimpyPoint, WimpyPointRect, app::{WimpyIO, wam::HardAsset, ImageData, EngineTextures, graphics::{GraphicsProvider, constants}}};

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
    destination:        WimpyTextureKey, // may need to be changed to a WimpyTextureKey
    source_origin:      UWimpyPoint,
    source_size:        UWimpyPoint,
    destination_origin: UWimpyPoint
}

pub struct TextureCreationParameters {
    pub wam_id:         HardAsset,
    /// A verified size or a hint provided by WAM.
    /// 
    /// If provided by WAM, the asset's real size in storage may be in disagreement with the manifest
    pub size_hint:      UWimpyPoint,
    pub policy_hint:    StreamingHint,
    pub slice:          Option<WimpyPointRect>
}

#[derive(Copy,Clone,PartialEq,Eq,Hash)]
pub enum BindGroupIdentity {
    Anonymous,
    Known(NonZeroU32)
}

/// An online texture container for textures that are on the GPU
/// 
/// It's a handle to a handle
pub struct WimpyTextureInternal {
    /// The size of the texture according to WAM metadata or similiar
    size_hint:          UWimpyPoint,
    wam_id:             Option<HardAsset>,
    pub bind_group_id:  BindGroupIdentity,
    pub view:           Option<TextureView>,
    /// RGBA8 Representation of texture information. TODO: Determine if this is linear or gamma space
    local_data:         Option<Vec<u8>>,
    policy_hint:        StreamingHint,
}

struct FallbackTexture {
    id: BindGroupIdentity,
    view: TextureView
}

impl WimpyTextureInternal {
    fn create_render_target(
        graphics_provider: &GraphicsProvider,
        bind_group_id: BindGroupIdentity,
        size: UWimpyPoint
    ) -> Self {
        Self {
            size_hint: size,
            wam_id: None,
            bind_group_id,
            view: Some(create_texture_view(graphics_provider,TextureViewConfig {
                size,
                render_attachment: true,
                image_data: None,
            })),
            local_data: None,
            policy_hint: StreamingHint::Static,
        }
    }
}

pub struct BindGroupIdentityGenerator {
    counter: NonZeroU32
}

impl Default for BindGroupIdentityGenerator {
    fn default() -> Self {
        Self { counter: NonZeroU32::MIN }
    }
}

impl BindGroupIdentityGenerator {
    pub fn next(&mut self) -> BindGroupIdentity {
        let current_id = self.counter;
        match self.counter.checked_add(1) {
            Some(next_id) => {
                self.counter = next_id;
            },
            None => {
                log::warn!("Texture ID counter overflow! You're living in the wild west now...");
            },
        };
        BindGroupIdentity::Known(current_id)
    }
}

pub struct TextureViewConfig<'a> {
    size:               UWimpyPoint,
    /// Specify if this texture resource will ever be used as a render pass attachment
    render_attachment:  bool,
    image_data:         Option<&'a [u8]>
}

fn create_texture_view(
    graphics_provider: &GraphicsProvider,
    config: TextureViewConfig,
) -> TextureView {

    let mut usage_flags =
        TextureUsages::TEXTURE_BINDING |
        TextureUsages::COPY_DST |
        TextureUsages::COPY_SRC;

    if config.render_attachment {
        usage_flags |= TextureUsages::RENDER_ATTACHMENT;
    };

    let size = config.size.into();

    let texture = graphics_provider.get_device().create_texture(&wgpu::TextureDescriptor {
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,

        format: constants::INTERNAL_TEXTURE_FORMAT,

        usage: usage_flags,
        label: Some("Texture"),
        view_formats: &[],
    });

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    if let Some(data) = config.image_data {
        graphics_provider.get_queue().write_texture(
            TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 1,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            data,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row:  Some(4 * size.width),
                rows_per_image: Some(size.height),
            },
            size,
        );
    }

    view
}

pub struct TextureManager {
    pub cache:              TextureCache,
    /// Textures that are generated at runtime
    pub runtime_textures:   RuntimeTextures,
    /// Textures that are expected to exist in the engine's asset namespace.
    /// 
    /// These fallback to the internal missing texture if not found.
    pub engine_textures:    EngineTextures,
    /// Provides an infalliable source to a texture view
    fallback_texture:       FallbackTexture,
    bind_groups:            BindGroupCache,
    id_generator:           BindGroupIdentityGenerator,
    update_queue:           Vec<UpdateOperation>,
    streaming_policy:       StreamingPolicy,
    atlases:                SlotMap<TextureAtlasKey,TextureAtlas>, //TODO: Should this be an atlas?
}

enum UpdateOperation {
    /// Contact operation to stream or refresh texture time to live
    Refresh(WimpyTextureKey),
    CopyTextureToTexture(TextureCopyParameters)
}

pub struct TextureAtlasConfig {
    /// How big a slot is (e.g., 16 pixels)
    pub slot_size: u32,
    /// How many slots are in the horizontal and vertical dimension (`number of slots` == `slot_length * slot_length`)
    pub slot_length: u32
}

/// A set of built in, always available texture assets.
/// 
/// Note, however, they still require unwrapping.
pub struct RuntimeTextures {
    pub missing:            WimpyTexture,
    pub opaque_white:       WimpyTexture,
    pub opaque_black:       WimpyTexture,
    pub transparent_white:  WimpyTexture,
    pub transparent_black:  WimpyTexture,
}

struct MissingTexture;

impl MissingTexture {
    const COLOR_1: [u8;Self::BYTES_PER_PIXEL] = [182,0,205,255];
    const COLOR_2: [u8;Self::BYTES_PER_PIXEL] = [53,23,91,255];

    const SIZE:             usize = 64;
    const GRID_DIVISION:    usize = 4;
    const BYTES_PER_PIXEL:  usize = 4;
    const PIXEL_COUNT:      usize = Self::SIZE * Self::SIZE;
    const DATA_SIZE:        usize = Self::PIXEL_COUNT * 4;

    fn get_color(x: usize,y: usize) -> [u8;Self::BYTES_PER_PIXEL] {
        let column = x / Self::GRID_DIVISION;
        let row =    y / Self::GRID_DIVISION;

        let checker_pattern = (column + row) % 2 == 0;

        match checker_pattern {
            true =>  Self::COLOR_1,
            false => Self::COLOR_2
        }
    }

    pub fn create_data() -> [u8;Self::DATA_SIZE] { 
        let mut data: [u8;Self::DATA_SIZE] = [0;Self::DATA_SIZE];

        let bytes = Self::BYTES_PER_PIXEL;

        let mut i: usize = 0;
        while i < Self::PIXEL_COUNT {
            let x = i % Self::SIZE;
            let y = i / Self::SIZE;

            let color = Self::get_color(x,y);

            data[i * bytes + 0] = color[0];
            data[i * bytes + 1] = color[1];
            data[i * bytes + 2] = color[2];
            data[i * bytes + 3] = color[3];

            i += 1;
        }

        data
    }
}

struct RuntimeTextureGenerator<'a> {
    graphics_provider:  &'a GraphicsProvider,
    texture_cache:      &'a mut TextureCache,
    id_generator:       &'a mut BindGroupIdentityGenerator
}

impl RuntimeTextureGenerator<'_> {
    fn create(&mut self,data: &[u8],size: UWimpyPoint) -> WimpyTexture {
        let view = create_texture_view(self.graphics_provider,TextureViewConfig {
            size,
            render_attachment: false,
            image_data: Some(data),
        });
        let texture = WimpyTextureInternal {
            size_hint: size,
            wam_id: None,
            bind_group_id: self.id_generator.next(),
            view: Some(view),
            local_data: None,
            policy_hint: StreamingHint::Static,
        };
        let key = self.texture_cache.insert_keyless(texture);
        WimpyTexture {
            key,
            size,
            slice: WimpyPointRect::area_from_size(size),
        }
    }
}

impl TextureManager {
    pub fn new(
        graphics_provider:  &GraphicsProvider,
        texture_layout:     BindGroupLayout,
        streaming_policy:   StreamingPolicy
    ) -> Self {
        let mut cache = TextureCache::new();
        let mut id_generator = BindGroupIdentityGenerator::default();

        let missing_texture_data = &MissingTexture::create_data();

        let fallback_texture = {
            let size = MissingTexture::SIZE.into();
            let view = create_texture_view(graphics_provider,TextureViewConfig {
                size,
                render_attachment: false,
                image_data: Some(missing_texture_data),
            });
            FallbackTexture {
                id: id_generator.next(),
                view
            }
        };

        let runtime_textures = {
            let mut generator = RuntimeTextureGenerator {
                graphics_provider: graphics_provider,
                texture_cache: &mut cache,
                id_generator: &mut id_generator
            };
            RuntimeTextures {
                missing:            generator.create(missing_texture_data,  MissingTexture::SIZE.into()),
                opaque_white:       generator.create(&[255,255,255,255],    UWimpyPoint::ONE),
                opaque_black:       generator.create(&[0,0,0,255],          UWimpyPoint::ONE),
                transparent_white:  generator.create(&[255,255,255,0],      UWimpyPoint::ONE),
                transparent_black:  generator.create(&[0,0,0,0],            UWimpyPoint::ONE),
            }
        };

        let engine_textures = EngineTextures::from_placeholder(&runtime_textures.missing);

        Self {
            cache,
            id_generator,
            runtime_textures,
            engine_textures,
            fallback_texture,
            streaming_policy,
            bind_groups:        BindGroupCache::create(graphics_provider.get_device(),texture_layout),
            atlases:            SlotMap::with_capacity_and_key(ATLAS_START_QUANTITY),
            update_queue:       Vec::with_capacity(UPDATE_OPERATIONS_BUFFER_DEFAULT_SIZE),
        }
    }

    pub fn bind_wam_asset(&mut self,parameters: TextureCreationParameters) -> WimpyTexture {
        let texture = WimpyTextureInternal {
            size_hint:      parameters.size_hint,
            wam_id:         Some(parameters.wam_id),
            bind_group_id:  self.id_generator.next(),
            view:           None,
            local_data:     None,
            policy_hint:    parameters.policy_hint,
        };
        let key = self.cache.insert_keyless(texture);
        WimpyTexture {
            key,
            size: parameters.size_hint,
            slice: parameters.slice.unwrap_or_else(||WimpyPointRect::area_from_size(parameters.size_hint))
        }
    }

    pub fn create_static_gpu_texture(&mut self,graphics_provider: &GraphicsProvider,image_data: ImageData) -> WimpyTexture {
        let texture_view = create_texture_view(graphics_provider,TextureViewConfig {
            size: image_data.size,
            render_attachment: false,
            image_data: Some(&image_data.data)
        });
        let texture = WimpyTextureInternal {
            size_hint:      image_data.size,
            wam_id:         None,
            bind_group_id:  self.id_generator.next(),
            view:           Some(texture_view),
            local_data:     None,
            policy_hint:    StreamingHint::Static,
        };
        let key = self.cache.insert_keyless(texture);
        WimpyTexture {
            key,
            size: image_data.size,
            slice: WimpyPointRect::area_from_size(image_data.size),
        }
    }

    // // fn copy_texture_to_texture(&mut self,parameters: TextureCopyParameters) {
    // //     self.update_queue.push(UpdateOperation::CopyTextureToTexture(parameters));
    // // }

    /// Refreshes time to live or restreams the texture to a gpu container
    fn refresh(&mut self,texture_key: WimpyTextureKey) {
        self.update_queue.push(UpdateOperation::Refresh(texture_key));
    }

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
    ) -> TextureAtlas {
        //todo: validate size with graphics provider
        todo!()
    }

    pub fn create_keyless_render_target(
        &mut self,
        graphics_provider: &GraphicsProvider,
        size: UWimpyPoint,
    ) -> WimpyTextureKey {
        let texture = WimpyTextureInternal::create_render_target(graphics_provider,self.id_generator.next(),size);
        self.cache.insert_keyless(texture)
    }

    pub fn borrow_render_target(
        &mut self,
        graphics_provider: &GraphicsProvider,
        size: UWimpyPoint,
        key: u32,
    ) -> WimpyTextureKey {
        match self.cache.start_lease(key) {
            Ok(value) => value, 
            Err(error) => {
                log::warn!("Graphics context creating a new temp frame. Reason: {:?}",error);
                let texture = WimpyTextureInternal::create_render_target(graphics_provider,self.id_generator.next(),size);
                self.cache.insert_with_lease(key,texture)
            },
        }
    }

    pub fn ensure_cached_render_target(
        &mut self,
        graphics_provider: &GraphicsProvider,
        size: UWimpyPoint,
        key: u32,
    ) {
        if self.cache.has_available_items(key) {
            return;
        }
        let texture = WimpyTextureInternal::create_render_target(graphics_provider,self.id_generator.next(),size);
        self.cache.insert(key,texture);
    }

    pub fn bind_output_surface(
        &mut self,
        surface: &SurfaceTexture,
        texture_view_format: TextureFormat,
    ) -> WimpyTextureKey {
        let texture_view = surface.texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Output Surface Texture View"),
            format: Some(texture_view_format),
            ..Default::default()
        });
        let texture = WimpyTextureInternal {
            size_hint: surface.texture.size().into(),
            wam_id: None,
            bind_group_id: BindGroupIdentity::Anonymous,
            view: Some(texture_view),
            local_data: None,
            policy_hint: StreamingHint::Static,
        };
        self.cache.insert_keyless(texture)
    }

    // This return values lives as long as 'self', not texture
    pub fn get_gpu_entry<'a,T>(&'a mut self,texture: &T) -> TextureCacheEntry<'a>
    where
        T: TextureCacheResolver
    {
        texture.get_entry(self)
    }

    // TODO: Bind group cache entries much be attached to their GPU textures otherwise they will leak GPU resources when dropping textures is intended
    pub fn get_bind_group_single_channel<'a>(&'a mut self,device: &Device,channel: BindGroupChannelConfig) -> &'a BindGroup {
        let (texture_view, id): (&TextureView, BindGroupIdentity) = match self.cache.get(channel.texture_key) {
            Ok(WimpyTextureInternal { view: Some(view), bind_group_id, .. }) => (view,*bind_group_id),
            _ => (&self.fallback_texture.view,self.fallback_texture.id),
        };
        self.bind_groups.get(device,&BindGroupChannelSet::Single { ch_0: BindGroupChannel { sampler_mode: channel.sampler_mode, texture_view, id }})
    }

    pub fn get_bind_group_dual_channel<'a>(&'a mut self,device: &Device,channels: [BindGroupChannelConfig;2]) -> &'a BindGroup {
        let [ch_0, ch_1] = channels.map(|channel|{
            let (texture_view, id): (&TextureView, BindGroupIdentity) = match self.cache.get(channel.texture_key) {
                Ok(WimpyTextureInternal { view: Some(view), bind_group_id, .. }) => (view,*bind_group_id),
                _ => (&self.fallback_texture.view,self.fallback_texture.id),
            };
            BindGroupChannel { sampler_mode: channel.sampler_mode, texture_view, id }
        });
        self.bind_groups.get(device,&BindGroupChannelSet::Dual { ch_0, ch_1 })
    }
}

/// Bind group cache channel configuration, an entry of a larger channel set
pub struct BindGroupChannelConfig {
    pub sampler_mode:   SamplerMode,
    pub texture_key:    WimpyTextureKey,
}

impl TextureCacheResolver for WimpyTextureKey {
    fn get_entry<'a>(&self,texture_manager: &'a mut TextureManager) -> TextureCacheEntry<'a> {
        match texture_manager.cache.get(*self) {
            Ok(WimpyTextureInternal { view: Some(view), size_hint, .. }) => {
                TextureCacheEntry {
                    input_size: *size_hint,
                    key: *self,
                    view,
                }
            },
            _ => {
                let fallback_texture = texture_manager.runtime_textures.missing;
                TextureCacheEntry {
                    input_size: fallback_texture.size,
                    key: fallback_texture.key,
                    view: &texture_manager.fallback_texture.view,
                }
            }
        }
    }
}

impl TextureCacheResolver for WimpyTexture {
    fn get_entry<'a>(&self,texture_manager: &'a mut TextureManager) -> TextureCacheEntry<'a> {
        self.key.get_entry(texture_manager)
    }
}

impl<T> TextureCacheResolver for T
where
    T: render_targets::RenderTarget
{
    fn get_entry<'a>(&self,texture_manager: &'a mut TextureManager) -> TextureCacheEntry<'a> {
        self.get_key().get_entry(texture_manager)
    }
}
