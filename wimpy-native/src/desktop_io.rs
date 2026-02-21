use std::path::Path;

use image::{
    DynamicImage, EncodableLayout, ImageError, ImageReader
};

use wimpy_engine::{UWimpyPoint, app::*};
use wimpy_engine::app::graphics::{
    TextureData,
    TextureDataWriteParameters
};

pub struct DekstopAppIO;

struct DynamicImageWrapper {
    value: DynamicImage
}

impl TextureData for DynamicImageWrapper {

    fn size(&self) -> UWimpyPoint {
        [self.value.width(),self.value.height()].into()
    }
    
    fn write_to_queue(self,parameters: &TextureDataWriteParameters) {
        parameters.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: parameters.texture,
                mip_level: parameters.mip_level,
                origin: parameters.origin,
                aspect: parameters.aspect,
            },
            self.value.to_rgba8().as_bytes(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                /* 1 byte per color in 8bit 4 channel color (RGBA with u8) */
                bytes_per_row: Some(self.value.width() * 4), 
                rows_per_image: Some(self.value.height()),
            },
            parameters.texture_size,
        );
    }
    
    fn get_format(&self) -> wgpu::TextureFormat {
        return wgpu::TextureFormat::Rgba8UnormSrgb;
    }
}

fn map_std_io_error(error: std::io::ErrorKind) -> FileError {
    use std::io::ErrorKind::*;
    return match error {
        NotFound => FileError::NotFound,
        PermissionDenied => FileError::NoPermission,
        ConnectionRefused => FileError::NoPermission,
        AddrNotAvailable => FileError::NotFound,
        InvalidFilename => FileError::InvalidPath,
        _ => FileError::Other,
    }
}

impl WimpyIO for DekstopAppIO {
    async fn load_image_file(path: &Path) -> Result<impl TextureData + 'static,FileError> {
        match ImageReader::open(path) {
            Ok(image_reader) => match image_reader.decode() {
                Ok(value) => {
                    Ok(DynamicImageWrapper { value })
                },
                Err(image_error) => Err(match image_error {
                    ImageError::Decoding(decoding_error) => {
                        log::error!("Image decode error '{:?}': {:?}",path,decoding_error);
                        FileError::DecodeFailure
                    },
                    ImageError::Unsupported(unsupported_error) => {
                        log::error!("Image unsupported error '{:?}': {:?}",path,unsupported_error);
                        FileError::DecodeFailure
                    },
                    ImageError::IoError(error) => {
                        //log::error!("Image IO error: {:?}",error);
                        map_std_io_error(error.kind())
                    },
                    _ => FileError::Unknown
                }),
            },
            Err(error) => Err({
                log::error!("Image IO error '{:?}': {}",path,error);
                map_std_io_error(error.kind())
            }),
        }
    }

    async fn load_binary_file(path: &Path) -> Result<Vec<u8>,FileError> {
        match std::fs::read(path) {
            Ok(value) => Ok(value),
            Err(error) => {
                //log::error!("Load binary file error ({:?}): {:?}",path,error);
                Err(map_std_io_error(error.kind()))
            }
        }
    }

    async fn load_text_file(path: &Path) -> Result<String,FileError> {
        match std::fs::read_to_string(path) {
            Ok(value) => Ok(value),
            Err(error) => {
                //log::error!("Load text file error ({:?}): {:?}",path,error);
                Err(map_std_io_error(error.kind()))
            }
        }
    }

    async fn save_key_value_store(data: &[u8]) -> Result<(),FileError> {
        todo!()
    }

    async fn load_key_value_store() -> Result<Vec<u8>,FileError> {
        todo!()
    }
}
