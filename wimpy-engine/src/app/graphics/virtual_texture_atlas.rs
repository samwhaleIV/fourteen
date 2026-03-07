use std::collections::HashMap;
use wgpu::{CommandEncoder, Extent3d, Origin3d, TexelCopyTextureInfo, TextureAspect};
use crate::{UWimpyPoint, WimpyRect, WimpyVec};
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
    /// The surface provided to a shader
    texture_container: TextureContainer,
    /// The indices available for use in the virtual atlas grid
    free_slot_ids: Vec<u16>,
    /// Textures currently rendered onto the `output_frame` surface
    bound_textures: HashMap<FrameCacheReference,AtlasSlotData>,

    /// The UV scalar to apply to atlas slots.
    /// 
    /// Represented as a reciprocal in order to avoid the expense of division (compared to multiplication) and divide by zero.
    size_recip: WimpyVec
}

struct AtlasSlotData {
    slot_id: u16,
    uv_area: WimpyRect
}

pub enum AtlasUpdate {
    AddTexture(FrameCacheReference),
    RemoveTexture(FrameCacheReference)
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

        let free_slots: Vec<u16> = (0..config.slot_length.pow(2) as u16).collect();
        let active_textures = HashMap::with_capacity(free_slots.len());

        Self {
            slot_size: config.slot_size,
            slot_length: pixel_size / config.slot_size,
            texture_container,
            free_slot_ids: free_slots,
            bound_textures: active_textures,
            size_recip: WimpyVec::ONE / WimpyVec::from(pixel_size)
        }
    }

    fn add_texture(&mut self,graphics_context: &mut GraphicsContext,encoder: &mut CommandEncoder,texture: FrameCacheReference) {
        let std::collections::hash_map::Entry::Vacant(entry) = self.bound_textures.entry(texture) else {
            return;
        };
        let Some(slot_id) = self.free_slot_ids.pop() else {
            log::warn!("Texture atlas is full; cannot insert texture");
            return;
        };
        let source_texture = match graphics_context.frame_cache.get(texture) {
            Ok(container) => {
                container.get_texture()
            },
            Err(_) => {
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
        let origin = get_slot_origin(self.slot_length,slot_id as u32);
        let dst = TexelCopyTextureInfo {
            texture: self.texture_container.get_texture(),
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
        entry.insert(AtlasSlotData {
            slot_id,
            uv_area: WimpyRect {
                position: WimpyVec::from(origin) * self.size_recip,
                size: WimpyVec::from(src_size) * self.size_recip,
            },
        });
    }

    fn remove_texture(&mut self,texture: FrameCacheReference) {
        let Some(slot_data) = self.bound_textures.remove(&texture) else {
            return;
        };
        self.free_slot_ids.push(slot_data.slot_id);
    }

    pub fn update<I>(&mut self,graphics_context: &mut GraphicsContext,encoder: &mut CommandEncoder,updates: I)
    where
        I: IntoIterator<Item = AtlasUpdate>
    {
        for update in updates.into_iter() {
            match update {
                AtlasUpdate::AddTexture(texture) => {
                    self.add_texture(graphics_context,encoder,texture)
                },
                AtlasUpdate::RemoveTexture(texture) => {
                    self.remove_texture(texture)
                },
            }
        }
    }

    pub fn get_uv_area(&self,texture: &FrameCacheReference) -> Option<WimpyRect> {
        let Some(slot_data) = self.bound_textures.get(&texture) else {
            // Texture not in atlas
            return None;
        };
        Some(slot_data.uv_area)
    }
}
