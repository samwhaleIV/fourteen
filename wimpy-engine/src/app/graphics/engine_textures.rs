use crate::{UWimpyPoint, app::wam::WimpyTexture};

use super::*;

const OPAQUE_BLACK: &[u8;4] = &[0,0,0,255];
const OPAQUE_WHITE: &[u8;4] = &[255,255,255,255];
const TRANSPARENT_WHITE: &[u8;4] = &[255,255,255,0];
const TRANSPARENT_BLACK: &[u8;4] = &[0,0,0,0];

struct TextureFrameBuilder<'a> {
    graphics_provider: &'a GraphicsProvider,
    texture_cache: &'a mut TextureCache,
    id_generator: &'a mut TextureIdentityGenerator
}

impl TextureFrameBuilder<'_> {
    fn create(&mut self,size: UWimpyPoint,data: &[u8]) -> WimpyTexture {
        let texture_container = TextureContainer::from_image_unchecked(
            self.graphics_provider,
            self.id_generator.next(),
            size,
            data
        );
        self.texture_cache.create_static_gpu_texture(texture_container)
    }
    fn create_pixel(&mut self,data: &[u8;4]) -> WimpyTexture {
        let size = UWimpyPoint::ONE;
        let texture_container = TextureContainer::from_image_unchecked(
            self.graphics_provider,
            self.id_generator.next(),
            size,
            data
        );
        self.texture_cache.create_static_gpu_texture(texture_container)
    }
}

impl EngineTextures {
    pub fn create(
        graphics_provider: &GraphicsProvider,
        id_generator: &mut TextureIdentityGenerator,
        texture_cache: &mut TextureCache,
    ) -> Self {

        let mut builder = TextureFrameBuilder {
            graphics_provider,
            id_generator,
            texture_cache,
        };

        let missing_texture = builder.create(
            UWimpyPoint::from(MissingTexture::SIZE),
            &MissingTexture::create().data
        );

        return Self {
            missing: missing_texture,
            opaque_white: builder.create_pixel(OPAQUE_WHITE),
            opaque_black: builder.create_pixel(OPAQUE_BLACK),
            transparent_white: builder.create_pixel(TRANSPARENT_WHITE),
            transparent_black: builder.create_pixel(TRANSPARENT_BLACK),

            font_classic: missing_texture.clone(),
            font_classic_outline: missing_texture.clone(),
            font_twelven: missing_texture.clone(),
            font_twelven_shaded: missing_texture.clone(),
            font_mono_elf: missing_texture.clone(),
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
