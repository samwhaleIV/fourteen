use super::*;

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
    fn size(&self) -> (u32,u32) {
        return (Self::SIZE as u32,Self::SIZE as u32);
    }
    fn get_format(&self) -> TextureFormat {
        return TextureFormat::Rgba8UnormSrgb;
    }
}
