use wgpu::{CommandEncoder, Origin3d, TexelCopyTextureInfo, TextureAspect};
use crate::{UWimpyPoint, WimpyRect, WimpyVec, collections::clock_cache::ClockCache};
use super::{WimpyTextureKey, BindGroupIdentity, TextureManager, SizeInfo};

pub struct TextureAtlas {
    /// How many slots occupy a dimension
    slot_length: u32,

    /// The size in pixels of a slot. An atlas item can be smaller than this squared, even rectangular, but cannot exceed this value in either dimension.
    slot_size: u32,

    /// The UV scalar to apply to atlas slots.
    /// 
    /// Represented as a reciprocal in order to avoid the expense of division (compared to multiplication) and divide by zero.
    size_recip: WimpyVec,

    pub bind_group_id: BindGroupIdentity,
    pub key: WimpyTextureKey,

    /// Backend cache for key/ownership logisitics
    /// 
    /// Does not contain cache values, only provides feedback for coordinated movements (inserted, dropped, or maintained)
    residency_cache: ClockCache<WimpyTextureKey>,

    encoder_commands: Vec<EncoderTextureCopyCommand>,

    /// A cache of sub-UV areas within the atlas surface
    metadata_cache: Vec<MetadataEntry>,
}

#[derive(Default,Copy,Clone,Eq,PartialEq)]
enum TextureCondition {
    #[default]
    Uninitialized,
    Placeholder(WimpyTextureKey),
    Loaded(WimpyTextureKey)
}

#[derive(Default,Copy,Clone)]
struct MetadataEntry {
    uv_area: WimpyRect,
    condition: TextureCondition
}

struct EncoderTextureCopyCommand {
    key:        WimpyTextureKey,
    src_size:   UWimpyPoint,
    dst_origin: UWimpyPoint,
}

impl TextureAtlas {
    pub fn new(
        slot_length: u32,
        slot_size: u32,
        texture_key: WimpyTextureKey,
        bind_group_id: BindGroupIdentity,
    ) -> Self {
        let slot_count = slot_size.pow(2) as usize;
        Self {
            slot_length,
            slot_size,
            key: texture_key,
            bind_group_id,
            size_recip: WimpyVec::ONE / WimpyVec::from(slot_size),
            metadata_cache: vec![Default::default();slot_count],
            residency_cache: ClockCache::new(slot_count),
            encoder_commands: Vec::with_capacity(slot_count / 4)
        }
    }

    /// Execute batched update commands for use with `encoder.copy_texture_to_texture`. The update command buffer is also drained.
    pub fn flush(
        &mut self,
        texture_manager: &mut TextureManager,
        encoder: &mut CommandEncoder
    ) {
        let dst_texture = match texture_manager.get_readonly(self.key) {
            Ok(texture) => texture.view.texture(),
            Err(error) => {
                log::warn!("Failure to retrieve atlas destination texture: {:?}",error);
                return;
            }
        };
        for command in self.encoder_commands.drain(..) {
            let src_texture = match texture_manager.get_readonly(command.key) {
                Ok(texture) => texture.view.texture(),
                Err(error) => {
                    log::warn!("Failure to retrieve atlas source texture: {:?}",error);
                    continue;
                }
            };
            let src = TexelCopyTextureInfo {
                texture:    src_texture,
                origin:     Origin3d::ZERO,
                aspect:     TextureAspect::All,
                mip_level:  0,
            };
            let dst = TexelCopyTextureInfo {
                texture:    dst_texture,
                origin:     command.dst_origin.into(),
                aspect:     TextureAspect::All,
                mip_level:  0,
            };
            encoder.copy_texture_to_texture(src,dst,command.src_size.into());
        }
    }

    fn get_slot_origin(&self,slot_id: u32) -> UWimpyPoint {
        UWimpyPoint {
            x: slot_id % self.slot_length * self.slot_size,
            y: slot_id / self.slot_length * self.slot_size,
        }
    }

    fn set_texture_internal(
        &mut self,
        texture_manager: &mut TextureManager,
        src_texture_key: WimpyTextureKey,
        cache_slot:      usize,
    ) {
        let src_texture = texture_manager.get(src_texture_key);

        let slot_size = self.slot_size;
        let mut src_size = UWimpyPoint::from(src_texture.size());

        if src_size.x > slot_size {
            log::warn!("Texture width '{}' too big for atlas slot '{}'",src_size.x,slot_size);
            src_size.x = src_size.x.min(slot_size);
        }

        if src_size.y > slot_size {
            log::warn!("Texture height '{}' too big for atlas slot '{}'",src_size.y,slot_size);
            src_size.y = src_size.y.min(slot_size);
        }

        let dst_origin = self.get_slot_origin(cache_slot as u32);

        self.encoder_commands.push(EncoderTextureCopyCommand {
            key: src_texture_key,
            src_size,
            dst_origin,
        });

        let uv_area = {
            let position = WimpyVec::from(dst_origin);
            let size =     WimpyVec::from(src_size);
            WimpyRect { position, size } * self.size_recip
        };

        let condition = match (self.metadata_cache[cache_slot].condition,src_texture.is_placeholder_view) {
            (TextureCondition::Uninitialized, true) => todo!(),
            (TextureCondition::Uninitialized, false) => todo!(),
            (TextureCondition::Placeholder(wimpy_texture_key), true) => todo!(),
            (TextureCondition::Placeholder(wimpy_texture_key), false) => todo!(),
            (TextureCondition::Loaded, true) => todo!(),
            (TextureCondition::Loaded, false) => todo!(),
        };

        self.metadata_cache[cache_slot] = MetadataEntry { uv_area, condition };
    }

    pub fn set_texture(
        &mut self,
        texture_manager: &mut TextureManager,
        src_texture_key: WimpyTextureKey
    ) -> WimpyRect {
        let cache_update = self.residency_cache.insert(src_texture_key);
        
        if cache_update.feedback.is_some() || src_texture_key {
            self.set_texture_internal(texture_manager,src_texture_key,cache_update.slot);
        }
        self.metadata_cache[cache_update.slot]
    }

    pub fn get_uv_area(&self,texture: WimpyTextureKey) -> Option<WimpyRect> {
        if let Some(slot) = self.residency_cache.get_slot_for_key(texture) {
            self.metadata_cache.get(slot).copied()
        } else {
            None
        }
    }
}
