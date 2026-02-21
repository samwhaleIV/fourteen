use crate::UWimpyPoint;

use super::*;
use wgpu::*;

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
            font_mono_elf: None,
        }
    }
}

pub struct MissingTexture {
    data: [u8;Self::DATA_SIZE]
}

impl MissingTexture {
    const COLOR_1: [u8;Self::BYTES_PER_PIXEL] = [182,0,205,255];
    const COLOR_2: [u8;Self::BYTES_PER_PIXEL] = [53,23,91,255];

    const SIZE: usize = 64;
    const GRID_DIVISION: usize = 4;
    const BYTES_PER_PIXEL: usize = 4;
    const PIXEL_COUNT: usize = Self::SIZE * Self::SIZE;
    const DATA_SIZE: usize = Self::PIXEL_COUNT * 4;

    fn get_color(x: usize,y: usize) -> [u8;Self::BYTES_PER_PIXEL] {
        let column = x / Self::GRID_DIVISION;
        let row = y / Self::GRID_DIVISION;

        let checker_pattern = (column + row) % 2 == 0;

        return match checker_pattern {
            true => Self::COLOR_1,
            false => Self::COLOR_2
        };
    }

    pub fn create() -> Self { 
        let mut data: [u8;Self::DATA_SIZE] = [0;Self::DATA_SIZE];

        let mut i: usize = 0;
        while i < Self::PIXEL_COUNT {
            let x = i % Self::SIZE;
            let y = i / Self::SIZE;

            let color = Self::get_color(x,y);

            data[i * Self::BYTES_PER_PIXEL + 0] = color[0];
            data[i * Self::BYTES_PER_PIXEL + 1] = color[1];
            data[i * Self::BYTES_PER_PIXEL + 2] = color[2];
            data[i * Self::BYTES_PER_PIXEL + 3] = color[3];

            i += 1;
        }

        return Self {
            data
        }
    }
}

impl TextureData for MissingTexture {
    fn write_to_queue(self,parameters: &TextureDataWriteParameters) {
        parameters.queue.write_texture(
            TexelCopyTextureInfo {
                texture: parameters.texture,
                mip_level: parameters.mip_level,
                origin: parameters.origin,
                aspect: parameters.aspect,
            },
            &self.data,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(Self::SIZE as u32 * 4), 
                rows_per_image: Some(Self::SIZE as u32),
            },
            parameters.texture_size,
        );
    }
    fn size(&self) -> UWimpyPoint {
        return Self::SIZE.into();
    }
    fn get_format(&self) -> TextureFormat {
        return TextureFormat::Rgba8UnormSrgb;
    }
}

pub struct OpaqueBlack;
pub struct OpaqueWhite;
pub struct TransparentWhite;
pub struct TransparentBlack;

trait GetColor {
    fn get_color() -> &'static [u8;4];
}

impl GetColor for OpaqueBlack {
    fn get_color() -> &'static [u8;4] {
        return &[0,0,0,255];
    }
}

impl GetColor for OpaqueWhite {
    fn get_color() -> &'static [u8;4] {
        return &[255,255,255,255];
    }
}

impl GetColor for TransparentWhite {
    fn get_color() -> &'static [u8;4] {
        return &[255,255,255,0];
    }
}

impl GetColor for TransparentBlack {
    fn get_color() -> &'static [u8;4] {
        return &[0,0,0,0];
    }
}

impl<T: GetColor> TextureData for T {
    fn write_to_queue(self,parameters: &TextureDataWriteParameters) {
        parameters.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: parameters.texture,
                mip_level: parameters.mip_level,
                origin: parameters.origin,
                aspect: parameters.aspect,
            },
            T::get_color(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4), 
                rows_per_image: Some(1),
            },
            parameters.texture_size,
        )
    }
    fn size(&self) -> UWimpyPoint {
        return [1,1].into()
    }
    fn get_format(&self) -> TextureFormat {
        return TextureFormat::Rgba8Unorm;
    }
}
