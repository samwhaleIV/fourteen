use super::*;

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
    fn size(&self) -> (u32,u32) {
        return (1,1);
    }
    fn get_format(&self) -> TextureFormat {
        return TextureFormat::Rgba8Unorm;
    }
}
