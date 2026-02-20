mod missing_texture;
mod pixel_textures;

use super::*;

use missing_texture::*;
use pixel_textures::*;

struct TextureFrameBuilder<'a> {
    graphics_provider: &'a GraphicsProvider,
    frame_cache: &'a mut FrameCache,
    id_generator: &'a mut TextureIdentityGenerator
}

impl TextureFrameBuilder<'_> {
    fn create(&mut self,data: impl TextureData) -> TextureFrame {
        let texture_container = TextureContainer::from_image_unchecked(
            self.graphics_provider,
            self.id_generator.next(),
            data
        );
        return FrameFactory::create_texture(
            texture_container.size(),
            self.frame_cache.insert_keyless(texture_container)
        );
    }
}

impl EngineTextures {
    pub fn create(
        graphics_provider: &GraphicsProvider,
        id_generator: &mut TextureIdentityGenerator,
        frame_cache: &mut FrameCache,
    ) -> Self {

        let mut builder = TextureFrameBuilder {
            graphics_provider,
            id_generator,
            frame_cache,
        };

        let missing_texture = MissingTexture::create();

        return Self {
            missing: builder.create(missing_texture),
            opaque_white: builder.create(OpaqueWhite),
            opaque_black: builder.create(OpaqueBlack),
            transparent_white: builder.create(TransparentWhite),
            transparent_black: builder.create(TransparentBlack),
            font_classic: None,
            font_classic_outline: None,
            font_twelven: None,
            font_twelven_shaded: None,
        }
    }
}
