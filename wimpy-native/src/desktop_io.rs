use std::path::Path;

use image::{DynamicImage, ImageError, ImageReader};

use wimpy_engine::app::*;
pub struct DekstopAppIO;

struct DynamicImageWrapper {
    value: DynamicImage
}

fn map_std_io_error(error: std::io::ErrorKind) -> FileError {
    use std::io::ErrorKind;
    return match error {
        ErrorKind::NotFound =>             FileError::NotFound,
        ErrorKind::PermissionDenied =>     FileError::NoPermission,
        ErrorKind::ConnectionRefused =>    FileError::NoPermission,
        ErrorKind::AddrNotAvailable =>     FileError::NotFound,
        ErrorKind::InvalidFilename =>      FileError::InvalidPath,
        _ =>                               FileError::Other,
    }
}

impl WimpyIO for DekstopAppIO {
    async fn load_image_file(path: &Path) -> Result<ImageData,FileError> {
        match ImageReader::open(path) {
            Ok(image_reader) => match image_reader.decode() {
                Ok(value) => {
                    Ok(ImageData {
                        size: [value.width(),value.height()].into(),
                        data: value.to_rgba8().to_vec(),
                    })
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
