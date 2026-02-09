mod generic;
pub mod image_functions;
pub mod model_functions;
pub mod text_functions;

use std::{
    path::PathBuf,
    rc::Rc
};

const PATH_BUFFER_START_SIZE: usize = 64;

use crate::{
    WimpyFileError,
    wam::*,
    wgpu::*
};

pub struct AssetManager {
    manifest: WamManifest,
    model_cache: ModelCache,
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
    FileError(WimpyFileError),
    ModelImportError(ModelImportError),
    TextureImportError(TextureContainerError),
}

pub struct AssetManagerCreationData<'a> {
    pub graphics_provider: &'a GraphicsProvider,
    pub content_root: Option<PathBuf>,
    pub manifest: WamManifest
}

impl AssetManager {
    pub fn create_without_manifest<TConfig>(graphics_provider: &GraphicsProvider) -> Self
    where
        TConfig: GraphicsContextConfig
    {
        return Self {
            manifest: Default::default(),
            path_buffer: PathBuf::with_capacity(PATH_BUFFER_START_SIZE),
            model_cache: ModelCache::create(
                graphics_provider,
                TConfig::MODEL_CACHE_VERTEX_BUFFER_SIZE,
                TConfig::MODEL_CACHE_INDEX_BUFFER_SIZE
            ),
        }
    }

    pub fn create<TConfig>(data: AssetManagerCreationData) -> Self
    where
        TConfig: GraphicsContextConfig
    {
        return Self {
            manifest: data.manifest,
            path_buffer: data.content_root.unwrap_or_else(||
                PathBuf::with_capacity(PATH_BUFFER_START_SIZE)
            ),
            model_cache: ModelCache::create(
                data.graphics_provider,
                TConfig::MODEL_CACHE_VERTEX_BUFFER_SIZE,
                TConfig::MODEL_CACHE_INDEX_BUFFER_SIZE
            ),
        }
    }
}
