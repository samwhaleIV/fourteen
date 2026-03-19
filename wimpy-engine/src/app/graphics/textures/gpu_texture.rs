use wgpu::*;
use std::num::NonZeroU32;

use crate::{UWimpyPoint};
use crate::app::graphics::{GraphicsProvider, constants};

#[derive(Copy,Clone,PartialEq,Eq,Hash)]
pub enum GPUTextureIdentity {
    Anonymous,
    Known(NonZeroU32)
}

/// An online texture container for textures that are on the GPU
/// 
/// It's a handle to a handle
pub struct GPUTexture {
    pub identity:   GPUTextureIdentity,
    pub input_size: UWimpyPoint,
    pub view:       TextureView,
}

pub struct GPUTextureIdentityGenerator {
    counter: NonZeroU32
}

impl Default for GPUTextureIdentityGenerator {
    fn default() -> Self {
        Self { counter: NonZeroU32::MIN }
    }
}

impl GPUTextureIdentityGenerator {
    pub fn next(&mut self) -> GPUTextureIdentity {
        let current_id = self.counter;
        match self.counter.checked_add(1) {
            Some(next_id) => {
                self.counter = next_id;
            },
            None => {
                log::warn!("Texture ID counter overflow! You're living in the wild west now...");
            },
        };
        GPUTextureIdentity::Known(current_id)
    }
}

pub struct GPUTextureConfig {
    pub size:               UWimpyPoint,
    pub identity:           GPUTextureIdentity,
    pub render_target:      bool,
    pub with_queue_data:    bool,
}

impl GPUTexture {

    pub fn new(
        graphics_provider: &GraphicsProvider,
        config: GPUTextureConfig
    ) -> Self {

        #[cfg(not(target_arch = "wasm32"))]
        let usage_flags = {
            let mut flags = TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_SRC;
            if config.with_queue_data {
                flags |= TextureUsages::COPY_DST;
            }
            if config.render_target {
                //TODO: only add dst when requested
                flags |= TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_DST;
            }
            flags
        };

        //Explanation... https://github.com/gpuweb/gpuweb/issues/3357#issuecomment-1223400585
        // https://github.com/hansjm10/Idle-Game-Engine/issues/846
        #[cfg(target_arch = "wasm32")]
        let usage_flags = {
            // TODO: Determine if this is an internal render target. If it is, we don't need all these flags
            let mut flags = TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC;
            flags
        };

        let device = graphics_provider.get_device();
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: Extent3d {
                width: config.size.x,
                height: config.size.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,

            format: constants::INTERNAL_RENDER_TARGET_FORMAT,

            usage: usage_flags,
            label: Some("Texture"),
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            input_size: config.size,
            view,
            identity: config.identity
        }
    }

    pub fn get_view(&self) -> &TextureView {
        &self.view
    }

    pub fn get_texture(&self) -> &Texture {
        self.view.texture()
    }

    pub fn get_identity(&self) -> GPUTextureIdentity {
        self.identity
    }

    pub fn size(&self) -> UWimpyPoint {
        self.input_size
    }
}
