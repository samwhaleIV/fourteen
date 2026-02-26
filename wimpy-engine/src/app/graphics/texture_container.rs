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
                log::warn!("Texture ID counter overflow. How do you have the RAM to make millions of textures? Wrapping the counter...");
                self.counter = NonZeroU32::MIN;
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
    texture_format: TextureFormat
}

pub struct TextureDataWriteParameters<'a> {
    pub queue: &'a Queue,
    pub texture: &'a Texture,
    pub texture_size: Extent3d,
    pub aspect: TextureAspect,
    pub mip_level: u32,
    pub origin: Origin3d,
}

pub trait TextureData {
    fn write_to_queue(self,parameters: &TextureDataWriteParameters);
    fn size(&self) -> UWimpyPoint;
    fn get_format(&self) -> TextureFormat;
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
            let mut flags = TextureUsages::TEXTURE_BINDING;
            if parameters.with_queue_data {
                flags |= TextureUsages::COPY_DST;
            }
            if parameters.render_target {
                flags |= TextureUsages::RENDER_ATTACHMENT;
            }
            flags
        };

        //Explanation... https://github.com/gpuweb/gpuweb/issues/3357#issuecomment-1223400585
        // https://github.com/hansjm10/Idle-Game-Engine/issues/846
        #[cfg(target_arch = "wasm32")]
        let usage_flags = TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;

        let device = graphics_provider.get_device();
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,

            format: parameters.texture_format,

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

    pub fn get_view(&self) -> &TextureView {
        return &self.view;
    }

    pub fn get_identity(&self) -> TextureContainerIdentity {
        return self.identity;
    }

    pub fn create_render_target(
        graphics_provider: &GraphicsProvider,
        identity: TextureContainerIdentity,
        size: UWimpyPoint // Externally validated (in graphics context)
    ) -> TextureContainer {
        Self::create(graphics_provider,TextureCreationParameters {
            size,
            identity,
            texture_format: INTERNAL_RENDER_TARGET_FORMAT,
            with_queue_data: false,
            render_target: true
        })
    }

    pub fn from_image_unchecked(
        graphics_provider: &GraphicsProvider,
        identity: TextureContainerIdentity,
        texture_data: impl TextureData
    ) -> TextureContainer {
        let size = texture_data.size();

        let texture_container = Self::create(graphics_provider,TextureCreationParameters {
            size,
            identity,
            texture_format: texture_data.get_format(),
            with_queue_data: true,
            render_target: false
        });

        texture_data.write_to_queue(&TextureDataWriteParameters {
            queue: graphics_provider.get_queue(),
            texture: texture_container.view.texture(),
            texture_size: texture_container.size,
            aspect: TextureAspect::All,
            mip_level: 0,
            origin: Origin3d::ZERO
        });

        return texture_container;
    }

    pub fn from_image(
        graphics_provider: &GraphicsProvider,
        identity: TextureContainerIdentity,
        texture_data: impl TextureData
    ) -> Result<TextureContainer,TextureError> {
        graphics_provider.test_size(texture_data.size())?;

        return Ok(Self::from_image_unchecked(graphics_provider,identity,texture_data));
    }

    pub fn create_output(
        surface: &SurfaceTexture,
        texture_view_format: TextureFormat,
        size: UWimpyPoint // Externally validated (in graphics context)
    ) -> TextureContainer {
        let view = surface.texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Output Surface Texture View"),
            format: Some(texture_view_format),
            ..Default::default()
        });
        return TextureContainer {
            identity: TextureContainerIdentity::Anonymous,
            size: Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            },
            view
        };
    }

    pub fn size(&self) -> UWimpyPoint {
        [self.size.width,self.size.height].into()
    }
}
