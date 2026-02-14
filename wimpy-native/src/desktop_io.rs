use std::{fs, path::Path};

use image::{
    DynamicImage,
    ImageError,
    ImageReader
};

use wimpy_engine::app::*;
use wimpy_engine::app::graphics::{
    TextureData,
    TextureDataWriteParameters
};

pub struct DekstopAppIO;

struct DynamicImageWrapper {
    value: DynamicImage
}

impl TextureData for DynamicImageWrapper {

    fn size(&self) -> (u32,u32) {
        (self.value.width(),self.value.height())
    }
    
    fn write_to_queue(self,parameters: &TextureDataWriteParameters) {
        parameters.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: parameters.texture,
                mip_level: parameters.mip_level,
                origin: parameters.origin,
                aspect: parameters.aspect,
            },
            self.value.as_bytes(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                /* 1 byte per color in 8bit 4 channel color (RGBA with u8) */
                bytes_per_row: Some(self.value.width() * 4), 
                rows_per_image: Some(self.value.height()),
            },
            parameters.texture_size,
        );
    }
}

impl WimpyIO for DekstopAppIO {
    async fn load_image_file(path: &Path) -> Result<impl TextureData + 'static,FileError> {
        match ImageReader::open(path) {
            Ok(image_reader) => match image_reader.decode() {
                Ok(value) => Ok(DynamicImageWrapper { value }),
                Err(image_error) => Err(match image_error {
                    ImageError::Decoding(decoding_error) => {
                        log::error!("Image decode error: {:?}",decoding_error);
                        FileError::Decode
                    },
                    ImageError::Unsupported(unsupported_error) => {
                        log::error!("Image unsupported error: {:?}",unsupported_error);
                        FileError::UnsupportedFormat
                    },
                    ImageError::IoError(error) => {
                        log::error!("Image IO error: {:?}",error);
                        FileError::Access
                    },
                    _ => FileError::Unknown
                }),
            },
            Err(error) => Err({
                log::error!("IO error: {:?}",error);
                FileError::Access
            }),
        }
    }
    
    async fn save_file(path: &Path,data: &[u8])-> Result<(),FileError> {
        if let Err(error) = (|| -> std::io::Result<()> {
            fs::create_dir_all(path)?;
            fs::write(path,data)?;
            Ok(())
        })() {
            log::error!("Save binary file error ({:?}): {:?}",path,error);
            return Err(FileError::Access);
        }
        Ok(())
    }
    
    async fn load_binary_file(path: &Path) -> Result<Vec<u8>,FileError> {
        let data = match (|| -> std::io::Result<Vec<u8>> {
            std::fs::create_dir_all(path)?;
            Ok(std::fs::read(path)?)
        })() {
            Ok(value) => value,
            Err(error) => {
                log::error!("Load binary file error ({:?}): {:?}",path,error);
                return Err(FileError::Access);
            }
        };
        return Ok(data);
    }

    async fn load_text_file(path: &Path) -> Result<String,FileError> {
        let data = match (|| -> std::io::Result<String> {
            std::fs::create_dir_all(path)?;
            Ok(std::fs::read_to_string(path)?)
        })() {
            Ok(value) => value,
            Err(error) => {
                log::error!("Load text file error ({:?}): {:?}",path,error);
                return Err(FileError::Access);
            }
        };
        return Ok(data);
    }

    async fn save_key_value_store(kvs: &KeyValueStore) -> Result<(),FileError> {
        todo!()
    }
    
    async fn load_key_value_store(kvs: &mut KeyValueStore) -> Result<(),FileError> {
        todo!()
    }
}
