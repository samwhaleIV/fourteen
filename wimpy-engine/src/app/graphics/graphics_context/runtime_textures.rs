mod missing_texture;
mod pixel_textures;

use super::*;

use missing_texture::*;
use pixel_textures::*;

pub struct RuntimeTextures {
    pub missing: TextureFrame,
    pub opaque_white: TextureFrame,
    pub opaque_black: TextureFrame,
    pub transparent_white: TextureFrame,
    pub transparent_black: TextureFrame
}

struct TextureFrameBuilder<'a> {
    frame_cache: &'a mut FrameCache,
    graphics_provider: &'a GraphicsProvider,
    texture_layout: &'a BindGroupLayout,
}

impl TextureFrameBuilder<'_> {
    fn create(&mut self,data: &impl TextureData) -> TextureFrame {
        let texture_container = TextureContainer::from_image_unchecked(
            self.graphics_provider,
            self.texture_layout,
            data
        );
        return FrameFactory::create_texture(
            texture_container.size(),
            self.frame_cache.insert_keyless(texture_container)
        );
    }
}

impl RuntimeTextures {
    pub fn create(
        frame_cache: &mut FrameCache,
        graphics_provider: &GraphicsProvider,
        texture_layout: &BindGroupLayout,
    ) -> Self {

        let mut builder = TextureFrameBuilder {
            frame_cache,
            graphics_provider,
            texture_layout,
        };

        let missing_texture = MissingTexture::create();

        return Self {
            missing: builder.create(&missing_texture),
            opaque_white: builder.create(&OpaqueWhite),
            opaque_black: builder.create(&OpaqueBlack),
            transparent_white: builder.create(&TransparentWhite),
            transparent_black: builder.create(&TransparentBlack),
        }
    }
}
