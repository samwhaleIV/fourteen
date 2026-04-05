use wgpu::{CommandEncoder, Extent3d, Origin3d, TexelCopyTextureInfo, TextureAspect, TextureView};
use crate::{UWimpyPoint, WimpyRect, WimpyVec, collections::clock_cache::ClockCache};
use super::{WimpyTextureKey, TextureCache, BindGroupIdentity, WimpyTextureInternal};

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
    pub texture_view: TextureView,

    /// Backend cache for key/ownership logisitics
    /// 
    /// Does not contain cache values, only provides feedback for coordinated movements (inserted, dropped, or maintained)
    residency_cache: ClockCache<WimpyTextureKey>,

    encoder_commands: Vec<EncoderTextureCopyCommand>,

    /// A cache of sub-UV areas within the atlas surface
    uv_cache: Vec<WimpyRect>,
}

struct EncoderTextureCopyCommand {
    texture:    wgpu::Texture,
    src_size:   UWimpyPoint,
    dst_origin: UWimpyPoint,
}

impl TextureAtlas {
    pub fn new(
        slot_length: u32,
        slot_size: u32,
        texture_view: TextureView,
        bind_group_id: BindGroupIdentity,
    ) -> Self {
        let slot_count = slot_size.pow(2) as usize;
        Self {
            slot_length,
            slot_size,
            texture_view,
            bind_group_id,
            size_recip: WimpyVec::ONE / WimpyVec::from(slot_size),
            uv_cache: vec![Default::default();slot_count],
            residency_cache: ClockCache::new(slot_count),
            encoder_commands: Vec::with_capacity(slot_count / 4)
        }
    }

    pub fn flush(&mut self,encoder: &mut CommandEncoder) {
        for command in self.encoder_commands.drain(..) {
            let src = TexelCopyTextureInfo {
                texture: &command.texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            };
            let dst = TexelCopyTextureInfo {
                texture: self.texture_view.texture(),
                mip_level: 0,
                origin: Origin3d {
                    x: command.dst_origin.x,
                    y: command.dst_origin.y,
                    z: 0,
                },
                aspect: TextureAspect::All,
            };
            let copy_size = Extent3d {
                width:  command.src_size.x,
                height: command.src_size.y,
                depth_or_array_layers: 1,
            };
            encoder.copy_texture_to_texture(src,dst,copy_size);
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
        frame_cache: &TextureCache,
        texture: WimpyTextureKey,
        slot: usize,
    ) {
        let source_texture = match frame_cache.get(texture) {
            Ok(WimpyTextureInternal { view: Some(view), .. }) => view.texture(),
            _ => {
                // This is not similiar to where we would want to use the 'missing' texture. This is an internal structural issue
                log::warn!("Texture not found; it's not in the frame cache");
                return;
            },
        };

        let slot_size = self.slot_size;
        let mut src_size = UWimpyPoint::from(source_texture.size());

        if src_size.x > slot_size {
            log::warn!("Texture width '{}' too big for atlas slot '{}'",src_size.x,slot_size);
            src_size.x = src_size.x.min(slot_size);
        }

        if src_size.y > slot_size {
            log::warn!("Texture height '{}' too big for atlas slot '{}'",src_size.y,slot_size);
            src_size.y = src_size.y.min(slot_size);
        }

        let dst_origin = self.get_slot_origin(slot as u32);

        self.encoder_commands.push(EncoderTextureCopyCommand {
            texture: source_texture.clone(),
            src_size,
            dst_origin,
        });

        let uv_area = WimpyRect {
            position: WimpyVec::from(dst_origin),
            size: WimpyVec::from(src_size)
        } * self.size_recip;

        self.uv_cache[slot] = uv_area;
    }

    pub fn set_texture(&mut self,frame_cache: &TextureCache,texture: WimpyTextureKey) -> WimpyRect {
        let cache_update = self.residency_cache.insert(texture);
        if cache_update.feedback.is_some() {
            self.set_texture_internal(frame_cache,texture,cache_update.slot);
        }
        self.uv_cache[cache_update.slot]
    }

    pub fn get_uv_area(&self,texture: WimpyTextureKey) -> Option<WimpyRect> {
        if let Some(slot) = self.residency_cache.get_slot_for_key(texture) {
            self.uv_cache.get(slot).copied()
        } else {
            None
        }
    }
}
