pub mod text_functions;
pub mod model_functions;
pub mod image_functions;

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
    pub fn create_without_manifest() -> Self {
        return Self {
            manifest: Default::default(),
            path_buffer: PathBuf::with_capacity(PATH_BUFFER_START_SIZE),
        }
    }

    pub fn create(data: AssetManagerCreationData) -> Self {
        return Self {
            manifest: data.manifest,
            path_buffer: data.content_root.unwrap_or_else(||
                PathBuf::with_capacity(PATH_BUFFER_START_SIZE)
            ),
        }
    }
}
