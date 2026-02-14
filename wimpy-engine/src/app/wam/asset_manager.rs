mod text_functions;
mod model_functions;
mod image_functions;

pub use model_functions::*;
pub use image_functions::*;

mod generic;
use super::prelude::*;
use crate::app::FileError;

use crate::app::graphics::{
    TextureError,
    ModelError
};

pub struct AssetManager {
    manifest: WamManifest,
    path_buffer: PathBuf,
}

#[derive(Debug)]
pub enum AssetManagerError {
    VirtualAssetNotFound(Rc<str>),
    MissingHardAsset(Rc<str>),
    AssetNotLoaded(Rc<str>),
    MismatchedType {
        expected: HardAssetType,
        found: HardAssetType
    },
    FileError(FileError),
    ModelImportError(ModelError),
    TextureImportError(TextureError),
}

pub struct AssetManagerCreationData {
    pub content_root: Option<PathBuf>,
    pub manifest: WamManifest
}

impl AssetManager {

    pub async fn load_or_default<IO: WimpyIO>(manifest_path: Option<&Path>) -> Self {
        return match manifest_path {
            Some(path) => match IO::load_text_file(path).await {
                Ok(json_text) => match WamManifest::create(&json_text) {
                    Ok(manifest) => {
                        let mut path_buffer = PathBuf::from(path);
                        path_buffer.pop();
                        Self::create(AssetManagerCreationData {
                            content_root: Some(path_buffer),
                            manifest
                        })
                    },
                    Err(error) => {
                        log::error!("Could not parse manifest data '{:?}': {:?}",path,error);
                        Self::create_without_manifest()
                    },
                },
                Err(error) => {
                    log::error!("Could not load manifest file '{:?}': {:?}",path,error);
                    Self::create_without_manifest()
                },
            },
            None => Self::create_without_manifest(),
        };
    }

    fn create_without_manifest() -> Self {
        return Self {
            manifest: Default::default(),
            path_buffer: PathBuf::with_capacity(PATH_BUFFER_START_SIZE),
        }
    }

    fn create(data: AssetManagerCreationData) -> Self {
        return Self {
            manifest: data.manifest,
            path_buffer: data.content_root.unwrap_or_else(||
                PathBuf::with_capacity(PATH_BUFFER_START_SIZE)
            ),
        }
    }
}
