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
    ModelError,
    TextureFrame,
    ModelCacheReference,
    GraphicsContext,
    CollisionShape,
    RenderBufferReference
};

pub struct AssetManager {
    manifest: WamManifest,
    root: PathBuf,
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

fn get_full_path(root: &PathBuf,virtual_path: &str) -> PathBuf {
    let mut path_buffer = PathBuf::new();
    path_buffer.push(root);
    for component in virtual_path.split('/') {
        path_buffer.push(component);
    }
    return path_buffer;
}

impl AssetManager {

    pub async fn load_or_default<IO: WimpyIO>(manifest_path: Option<&Path>) -> Self {
        return match manifest_path {
            Some(path) => match IO::load_text_file(path).await {
                Ok(json_text) => match WamManifest::create(&json_text) {
                    Ok(manifest) => {
                        let mut path_buffer = PathBuf::from(path);
                        path_buffer.pop();
                        Self {
                            root: path_buffer,
                            manifest: manifest
                        }
                    },
                    Err(error) => {
                        log::error!("Could not parse manifest data '{:?}': {:?}",path,error);
                        Self::create_empty()
                    },
                },
                Err(error) => {
                    log::error!("Could not load manifest file '{:?}': {:?}",path,error);
                    Self::create_empty()
                },
            },
            None => Self::create_empty(),
        };
    }

    fn create_empty() -> Self {
        return Self {
            manifest: Default::default(),
            root: Default::default(),
        }
    }
}
