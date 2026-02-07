use std::fs;

use image::{
    DynamicImage,
    ImageError,
    ImageReader
};

use wimpy_engine::{
    WimpyFileError,
    WimpyIO,
    kvs::KeyValueStore,
    wgpu::{
        TextureData,
        TextureDataWriteParameters
    }
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
    async fn load_image(path: &str) -> Result<impl TextureData,WimpyFileError> {
        match ImageReader::open(path) {
            Ok(image_reader) => match image_reader.decode() {
                Ok(value) => Ok(DynamicImageWrapper { value }),
                Err(image_error) => Err(match image_error {
                    ImageError::Decoding(decoding_error) => {
                        log::error!("Image decode error: {:?}",decoding_error);
                        WimpyFileError::Decode
                    },
                    ImageError::Unsupported(unsupported_error) => {
                        log::error!("Image unsupported error: {:?}",unsupported_error);
                        WimpyFileError::UnsupportedFormat
                    },
                    ImageError::IoError(error) => {
                        log::error!("Image IO error: {:?}",error);
                        WimpyFileError::Access
                    },
                    _ => WimpyFileError::Unknown
                }),
            },
            Err(error) => Err({
                log::error!("IO error: {:?}",error);
                WimpyFileError::Access
            }),
        }
    }
    
    async fn save_file(path: &str,data: &[u8])-> Result<(),WimpyFileError> {
        if let Err(error) = (|| -> std::io::Result<()> {
            fs::create_dir_all(path)?;
            fs::write(path,data)?;
            Ok(())
        })() {
            log::error!("Save binary file error ({}): {:?}",path,error);
            return Err(WimpyFileError::Access);
        }
        Ok(())
    }
    
    async fn load_binary_file(path: &str) -> Result<Vec<u8>,WimpyFileError> {
        let data = match (|| -> std::io::Result<Vec<u8>> {
            std::fs::create_dir_all(path)?;
            Ok(std::fs::read(path)?)
        })() {
            Ok(value) => value,
            Err(error) => {
                log::error!("Load binary file error ({}): {:?}",path,error);
                return Err(WimpyFileError::Access);
            }
        };
        return Ok(data);
    }
    
    async fn load_text_file(path: &str) -> Result<String,WimpyFileError> {
        let data = match (|| -> std::io::Result<String> {
            std::fs::create_dir_all(path)?;
            Ok(std::fs::read_to_string(path)?)
        })() {
            Ok(value) => value,
            Err(error) => {
                log::error!("Load text file error ({}): {:?}",path,error);
                return Err(WimpyFileError::Access);
            }
        };
        return Ok(data);
    }
    
    async fn save_key_value_store(kvs: &KeyValueStore) -> Result<(),WimpyFileError> {
        todo!()
    }
    
    async fn load_key_value_store(kvs: &mut KeyValueStore) -> Result<(),WimpyFileError> {
        todo!()
    }
}
