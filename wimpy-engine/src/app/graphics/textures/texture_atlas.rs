use wgpu::{CommandEncoder, Extent3d, Origin3d, TexelCopyTextureInfo, TextureAspect};
use crate::{UWimpyPoint, WimpyRect, WimpyVec, collections::clock_cache::ClockCache};
use super::{WimpyTextureKey, BindGroupIdentity, TextureManager, TextureCacheEntry, TextureLoadState};

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
    cell_cache: Vec<Cell>,
}

#[derive(Default,Copy,Clone,PartialEq,Eq)]
enum CellCondition {
    #[default]
    Empty,
    Registered(WimpyTextureKey, TextureLoadState)
}

#[derive(Default,Copy,Clone)]
struct Cell {
    uv_area: WimpyRect,
    condition: CellCondition
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
        let slot_count = slot_length.pow(2) as usize;
        let atlas_width = slot_size * slot_length;
        Self {
            slot_length,
            slot_size,
            key: texture_key,
            bind_group_id,
            size_recip: WimpyVec::ONE / WimpyVec::from(atlas_width), 
            cell_cache: vec![Default::default();slot_count],
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
        let dst_texture = match texture_manager.get_no_touch(self.key) {
            Ok(texture) => texture.view.texture(),
            Err(error) => {
                log::warn!("Failure to retrieve atlas destination texture: {:?}",error);
                return;
            }
        };
        for command in self.encoder_commands.drain(..) {
            let src_texture = match texture_manager.get_no_touch(command.key) {
                Ok(entry) => entry,
                Err(_) => texture_manager.get_fallback_texture(command.key),
            };
            let src = TexelCopyTextureInfo {
                texture:    src_texture.view.texture(),
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
            encoder.copy_texture_to_texture(src, dst, Extent3d {
                width: command.src_size.x.min(src.texture.width()),
                height: command.src_size.y.min(src.texture.height()),
                depth_or_array_layers: 1,
            });
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
        src_texture: TextureCacheEntry,
        cache_slot:  usize,
    ) {
        let new_key = src_texture.key;
        let new_condition = CellCondition::Registered(new_key, src_texture.load_state);
        let mut src_size = src_texture.input_size;

        let slot_size = self.slot_size;
        if src_size.x > slot_size {
            log::warn!("Texture width '{}' too big for atlas slot '{}'",src_size.x,slot_size);
            src_size.x = src_size.x.min(slot_size);
        }

        if src_size.y > slot_size {
            log::warn!("Texture height '{}' too big for atlas slot '{}'",src_size.y,slot_size);
            src_size.y = src_size.y.min(slot_size);
        }

        let dst_origin = self.get_slot_origin(cache_slot as u32);

        let uv_area = { // UV area with a pixel of total inset and half a pixel of offset
            let position = WimpyVec {
                x: dst_origin.x as f32 + 0.5,
                y: dst_origin.y as f32 + 0.5,
            };
            let size = WimpyVec {
                x: (src_size.x as f32 - 1.0).max(0.0),
                y: (src_size.y as f32 - 1.0).max(0.0),
            };
            WimpyRect { position, size } * self.size_recip
        };

        self.encoder_commands.push(EncoderTextureCopyCommand {
            key: new_key,
            src_size,
            dst_origin,
        });

        self.cell_cache[cache_slot] = Cell { uv_area, condition: new_condition };
    }

    pub fn set_texture(
        &mut self,
        texture_manager: &mut TextureManager,
        src_texture_key: WimpyTextureKey
    ) -> WimpyRect {
        let src_texture = texture_manager.get_or_default(src_texture_key);
        let cache_update = self.residency_cache.insert(src_texture_key);

        let texture_status_changed = {
            if let Some(cell_state) = self.cell_cache.get(cache_update.slot) {
                // If this is NOT equal, the texture state changed (E.g., incremental load progress, failure, etc.)
                cell_state.condition != CellCondition::Registered(src_texture_key, src_texture.load_state)
            } else {
                true
            }
        };

        if cache_update.feedback.is_some() || texture_status_changed {
            self.set_texture_internal(src_texture, cache_update.slot);
        }
        
        self.cell_cache[cache_update.slot].uv_area
    }

    pub fn get_uv_area(&self,texture: WimpyTextureKey) -> Option<WimpyRect> {
        let Some(slot) = self.residency_cache.get_slot_for_key(texture) else {
            return None;
        };
        let Some(cell) = self.cell_cache.get(slot) else {
            // Should this return a placeholder value instead of none?
            return None;
        };
        return Some(cell.uv_area);
    }
}
