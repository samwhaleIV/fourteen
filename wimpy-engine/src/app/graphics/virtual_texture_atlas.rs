use wgpu::{CommandEncoder, Extent3d, Origin3d, TexelCopyTextureInfo, TextureAspect};
use crate::{UWimpyPoint, WimpyRect, WimpyVec, collections::ClockCache};
use super::{FrameCacheReference, GraphicsContext, TextureContainer, GraphicsProvider, TextureContainerIdentity};

pub struct VirtualTextureAtlasConfig {
    /// How big a slot is (e.g., 16 pixels)
    pub slot_size: u32,
    /// How many slots are in the horizontal and vertical dimension (`number of slots` == `slot_length * slot_length`)
    pub slot_length: u32
}

pub struct VirtualTextureAtlas {
    /// The size in pixels of a slot. An atlas item can be smaller than this squared, even rectangular, but cannot exceed this value in either dimension.
    slot_size: u32,

    /// How many slots occupy a dimension
    slot_length: u32,

    /// The UV scalar to apply to atlas slots.
    /// 
    /// Represented as a reciprocal in order to avoid the expense of division (compared to multiplication) and divide by zero.
    size_recip: WimpyVec,

    /// The surface provided to a shader
    atlas_texture: TextureContainer,

    /// Backend cache for key/ownership logisitics
    /// 
    /// Does not contain cache values, only provides feedback for coordinated movements (inserted, dropped, or maintained)
    residency_cache: ClockCache<FrameCacheReference>,

    /// A cache of sub-UV areas within the atlas surface
    uv_cache: Vec<WimpyRect>,
}

const fn get_slot_origin(slot_length: u32,slot_id: u32) -> UWimpyPoint {
    UWimpyPoint {
        x: slot_id % slot_length,
        y: slot_id / slot_length,
    }
}

impl VirtualTextureAtlas {
    pub fn create(
        graphics_provider: &GraphicsProvider,
        texture_identity: TextureContainerIdentity,
        config: &VirtualTextureAtlasConfig
    ) -> Self {

        let pixel_length = config.slot_length * config.slot_size;
        let pixel_size = graphics_provider.get_safe_texture_dimension_value(pixel_length);

        let texture_container = TextureContainer::create_render_target(
            graphics_provider,
            texture_identity, // Used so we can use the texture as a bind group target
            UWimpyPoint::from(pixel_size)
        );

        let slot_count = config.slot_size.pow(2) as usize;

        Self {
            slot_size: config.slot_size,
            slot_length: pixel_size / config.slot_size,
            atlas_texture: texture_container,
            size_recip: WimpyVec::ONE / WimpyVec::from(pixel_size),
            residency_cache: ClockCache::new(slot_count),
            uv_cache: vec![Default::default();slot_count]
        }
    }

    fn write_texture_to_surface(
        &mut self,
        graphics_context: &mut GraphicsContext,
        encoder: &mut CommandEncoder,
        texture: FrameCacheReference,
        slot: usize,
    ) {
        let source_texture = match graphics_context.frame_cache.get(texture) {
            Ok(container) => {
                container.get_texture()
            },
            Err(_) => {
                // This is not similiar to where we would want to use the 'missing' texture. This is an internal structural issue
                log::warn!("Texture not found; it's not in the frame cache");
                return;
            },
        };
        let src_size = source_texture.size();
        if
            src_size.width > self.slot_size ||
            src_size.height > self.slot_size
        {
            log::warn!("Texture too big for atlas slot");
            return;
        }
        let src = TexelCopyTextureInfo {
            texture: source_texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: TextureAspect::All,
        };
        let origin = get_slot_origin(self.slot_length,slot as u32);
        let dst = TexelCopyTextureInfo {
            texture: self.atlas_texture.get_texture(),
            mip_level: 0,
            origin: Origin3d {
                x: origin.x,
                y: origin.y,
                z: 0,
            },
            aspect: TextureAspect::All,
        };
        let copy_size = Extent3d {
            width: src_size.width,
            height: src_size.height,
            depth_or_array_layers: 1,
        };
        encoder.copy_texture_to_texture(src,dst,copy_size);

        if let Some(uv) = self.uv_cache.get_mut(slot) {
            uv.position = WimpyVec::from(origin) * self.size_recip;
            uv.size = WimpyVec::from(src_size) * self.size_recip;
        }
    }

    pub fn set_textures<I>(
        &mut self,graphics_context: &mut GraphicsContext,
        encoder: &mut CommandEncoder,
        textures: I
    )
    where
        I: IntoIterator<Item = FrameCacheReference>
    {
        for texture in textures.into_iter() {
            if let Some(cache_change) = self.residency_cache.insert(texture) {
                self.write_texture_to_surface(graphics_context,encoder,texture,cache_change.slot);
            }
        }
    }

    pub fn get_uv_area(&self,texture: &FrameCacheReference) -> Option<WimpyRect> {
        if let Some(slot) = self.residency_cache.get_slot_for_key(*texture) {
            self.uv_cache.get(slot).copied()
        } else {
            None
        }
    }
}
