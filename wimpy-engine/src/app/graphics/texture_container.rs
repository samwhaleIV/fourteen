use crate::UWimpyPoint;

use super::*;
use wgpu::*;
use std::num::NonZeroU32;
use super::constants::*;

#[derive(Copy,Clone,PartialEq,Eq,Hash)]
pub enum TextureContainerIdentity {
    Anonymous,
    Known(NonZeroU32)
}

/// An online texture container for textures that are on the GPU
pub struct TextureContainer {
    identity: TextureContainerIdentity,
    size: Extent3d,
    view: TextureView,
}

pub struct TextureIdentityGenerator {
    counter: NonZeroU32
}

impl Default for TextureIdentityGenerator {
    fn default() -> Self {
        Self {
            counter: NonZeroU32::MIN
        }
    }
}

impl TextureIdentityGenerator {
    pub fn next(&mut self) -> TextureContainerIdentity {
        let current_id = self.counter;
        match self.counter.checked_add(1) {
            Some(next_id) => {
                self.counter = next_id;
            },
            None => {
                log::warn!("Texture ID counter overflow. How do you have the RAM to make millions of textures?");
            },
        };
        return TextureContainerIdentity::Known(current_id);
    }
}

struct TextureCreationParameters {
    size: UWimpyPoint,
    identity: TextureContainerIdentity,
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    render_target: bool,
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    with_queue_data: bool,
    //texture_format: TextureFormat,
}

impl TextureContainer {

    fn create(
        graphics_provider: &GraphicsProvider,
        parameters: TextureCreationParameters
    ) -> Self {

        let size = wgpu::Extent3d {
            width: parameters.size.x,
            height: parameters.size.y,
            depth_or_array_layers: 1,
        };

        #[cfg(not(target_arch = "wasm32"))]
        let usage_flags = {
            let mut flags = TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_SRC;
            if parameters.with_queue_data {
                flags |= TextureUsages::COPY_DST;
            }
            if parameters.render_target {
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
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,

            format: INTERNAL_RENDER_TARGET_FORMAT,

            usage: usage_flags,
            label: Some("Texture"),
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        return TextureContainer {
            size,
            view,
            identity: parameters.identity
        };
    }

    pub fn get_texture_view(&self) -> &TextureView {
        &self.view
    }

    pub fn get_texture(&self) -> &Texture {
        self.view.texture()
    }

    pub fn get_identity(&self) -> TextureContainerIdentity {
        self.identity
    }

    pub fn create_render_target(
        graphics_provider: &GraphicsProvider,
        identity: TextureContainerIdentity,
        size: UWimpyPoint // Externally validated (in graphics context)
    ) -> Self {
        Self::create(graphics_provider,TextureCreationParameters {
            size,
            identity,
            with_queue_data: false,
            render_target: true,
        })
    }

    pub fn from_image_unchecked(
        graphics_provider: &GraphicsProvider,
        identity: TextureContainerIdentity,
        size: UWimpyPoint,
        data: &[u8]
    ) -> Self {
        let texture_container = Self::create(graphics_provider,TextureCreationParameters {
            size,
            identity,
            with_queue_data: true,
            render_target: false,
        });
        graphics_provider.get_queue().write_texture(
            TexelCopyTextureInfo {
                texture: texture_container.get_texture(),
                mip_level: 1,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            data,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * size.x),
                rows_per_image: Some(size.y),
            },
            Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            },
        );
        return texture_container;
    }

    pub fn from_image(
        graphics_provider: &GraphicsProvider,
        identity: TextureContainerIdentity,
        size: UWimpyPoint,
        data: &[u8]
    ) -> Result<Self,TextureError> {
        graphics_provider.validate_size(size)?;

        return Ok(Self::from_image_unchecked(graphics_provider,identity,size,data));
    }

    pub fn create_output(
        surface: &SurfaceTexture,
        texture_view_format: TextureFormat,
        size: UWimpyPoint // Externally validated (in graphics context)
    ) -> Self {
        let view = surface.texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Output Surface Texture View"),
            format: Some(texture_view_format),
            ..Default::default()
        });
        Self {
            identity: TextureContainerIdentity::Anonymous,
            size: Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            },
            view
        }
    }

    pub fn size(&self) -> UWimpyPoint {
        UWimpyPoint {
            x: self.size.width,
            y: self.size.height,
        }
    }
}
