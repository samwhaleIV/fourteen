use slotmap::SparseSecondaryMap;

use super::prelude::*;
use crate::app::{AssetLoadingContext, FileError};

use crate::app::graphics::{GraphicsContext, ModelError, TextureError, TextureFrame, TexturedMeshReference, TexturedMeshlet};

const START_CACHE_ENTRY_CAPACITY: usize = 8;

#[derive(Default)]
pub struct AssetManager {
    pub manifest: WamManifest,
    root: PathBuf,
    text_cache: SparseSecondaryMap<HardAssetKey,Rc<str>>,
    image_cache: SparseSecondaryMap<HardAssetKey,TextureFrameView>,
    model_cache: SparseSecondaryMap<HardAssetKey,TexturedMeshReference>,
}

#[derive(Copy,Clone)]
pub struct TextureFrameView {
    pub texture: TextureFrame,
    pub view: Option<ImageArea>
}

#[derive(Debug)]
pub enum AssetManagerError {
    VirtualAssetNotFound(&'static str),
    MissingHardAsset(&'static str),
    MismatchedType {
        expected: HardAssetType,
        found: HardAssetType
    },
    FileError(FileError),
    ModelImportError(ModelError),
    TextureImportError(TextureError),
}

fn get_full_path(root: &PathBuf,hard_asset_path: &str) -> PathBuf {
    let mut path_buffer = PathBuf::new();
    path_buffer.push(root);
    for component in hard_asset_path.split('/') {
        path_buffer.push(component);
    }
    return path_buffer;
}

fn validate_hard_asset_type(hard_asset: &HardAsset,expected_type: HardAssetType) -> Result<(),AssetManagerError> {
    if hard_asset.data_type != expected_type {
        Err(AssetManagerError::MismatchedType {
            expected: expected_type,
            found: hard_asset.data_type 
        })
    } else {
        Ok(())
    }
}

impl AssetManager {
    pub async fn load_or_default<IO: WimpyIO>(manifest_path: Option<&Path>) -> Self {
        match manifest_path {
            Some(path) => match IO::load_text_file(path).await {
                Ok(json_text) => match WamManifest::create(&json_text) {
                    Ok(manifest) => {
                        let mut path_buffer = PathBuf::from(path);
                        path_buffer.pop();
                        Self {
                            root: path_buffer,
                            manifest: manifest,
                            text_cache: SparseSecondaryMap::with_capacity(START_CACHE_ENTRY_CAPACITY),
                            image_cache: SparseSecondaryMap::with_capacity(START_CACHE_ENTRY_CAPACITY),
                            model_cache: SparseSecondaryMap::with_capacity(START_CACHE_ENTRY_CAPACITY),
                        }
                    },
                    Err(error) => {
                        log::error!("Could not parse manifest data '{:?}': {:?}",path,error);
                        Default::default()
                    },
                },
                Err(error) => {
                    log::error!("Could not load manifest file '{:?}': {:?}",path,error);
                    Default::default()
                },
            },
            None => Default::default(),
        }
    }

    async fn get_text_cached<IO: WimpyIO>(&mut self,key: HardAssetKey,name: &'static str) -> Result<Rc<str>,AssetManagerError> {
        if let Some(text) = self.text_cache.get(key) {
            return Ok(text.clone());
        }

        let hard_asset = match self.manifest.hard_assets.get(key) {
            Some(value) => value,
            None => return Err(AssetManagerError::MissingHardAsset(name)),
        };

        validate_hard_asset_type(hard_asset,HardAssetType::Text)?;

        let path = get_full_path(&self.root,&hard_asset.file_source);
        let text_data: Rc<str> = Rc::from(match IO::load_text_file(path.as_path()).await {
            Ok(data) => data,
            Err(error) => return Err(AssetManagerError::FileError(error)),
        });

        self.text_cache.insert(key,text_data.clone());

        Ok(text_data)
    }

    async fn get_image_cached<IO: WimpyIO>(
        &mut self,key: HardAssetKey,
        name: &'static str,
        texture_view: Option<ImageArea>,
        graphics_context: &mut GraphicsContext
    ) -> Result<TextureFrameView,AssetManagerError> {
        if let Some(image) = self.image_cache.get(key) {
            return Ok(image.clone());
        }

        let hard_asset = match self.manifest.hard_assets.get(key) {
            Some(value) => value,
            None => return Err(AssetManagerError::MissingHardAsset(name)),
        };

        validate_hard_asset_type(hard_asset,HardAssetType::Image)?;

        let path = get_full_path(&self.root,&hard_asset.file_source);

        let image_data = match IO::load_image_file(path.as_path()).await {
            Ok(data) => data,
            Err(error) => return Err(AssetManagerError::FileError(error)),
        };
        let texture_frame = match graphics_context.create_texture_frame(image_data) {
            Ok(value) => value,
            Err(error) => return Err(AssetManagerError::TextureImportError(error)),
        };

        let texture_frame_view = TextureFrameView {
            texture: texture_frame.clone(),
            view: texture_view
        };

        self.image_cache.insert(key,texture_frame_view.clone());

        Ok(texture_frame_view)
    }
}

pub trait UserAssetMapping {
    type UserAsset;
    fn get_cached<IO: WimpyIO>(name: &'static str,context: &mut AssetLoadingContext<'_>) -> impl Future<Output = Result<Self::UserAsset,AssetManagerError>>;
}

impl UserAssetMapping for ModelAssetReference {
    type UserAsset = TexturedMeshReference;

    async fn get_cached<IO: WimpyIO>(name: &'static str,context: &mut AssetLoadingContext<'_>) -> Result<Self::UserAsset,AssetManagerError> {

        let (hard_asset_key,meshlet_descriptors) = {
            let Some(virtual_asset) = context.asset_manager.manifest.model_assets.get(name) else {
                return Err(AssetManagerError::VirtualAssetNotFound(name));
            };
            // I was so profoundly pissed off by the borrow checker that I threw in a clone here
            (virtual_asset.key,virtual_asset.meshlet_descriptors.clone())
        };

        /* We can't use 'entry()' because we mutate the slotmap cache after this to get textures */
        if let Some(mesh) = context.asset_manager.model_cache.get(hard_asset_key) {
            return Ok(mesh.clone());
        }

        let hard_asset = match context.asset_manager.manifest.hard_assets.get(hard_asset_key) {
            Some(value) => value,
            None => return Err(AssetManagerError::MissingHardAsset(name)),
        };
        validate_hard_asset_type(hard_asset,HardAssetType::Model)?;

        let path = get_full_path(&context.asset_manager.root,&hard_asset.file_source);
        let gltf_data = match IO::load_binary_file(path.as_path()).await {
            Ok(data) => data,
            Err(error) => return Err(AssetManagerError::FileError(error)),
        };

        let queue = context.graphics_context.graphics_provider.get_queue();
        let mesh = match context.graphics_context.mesh_cache.insert_geometry(queue,&gltf_data) {
            Ok(value) => value,
            Err(error) => return Err(AssetManagerError::ModelImportError(error)),
        };

        // There may be more meshlet descriptions than meshlet geometry, or vice versa
        let limit = mesh.len().min(meshlet_descriptors.len());

        let mut textured_mesh: Vec<TexturedMeshlet> = Vec::with_capacity(limit);

        for (i,meshlet) in mesh.into_iter().enumerate() {
            let descriptor = &meshlet_descriptors[i];

            let diffuse = match descriptor.diffuse {
                Some(key) => context.asset_manager.get_image_cached::<IO>(key,&name,None,context.graphics_context).await?.texture,
                None => context.graphics_context.engine_textures.missing,
            };

            let lightmap = match descriptor.lightmap {
                Some(key) => context.asset_manager.get_image_cached::<IO>(key,&name,None,context.graphics_context).await?.texture,
                None => context.graphics_context.engine_textures.opaque_white,
            };

            textured_mesh.push(TexturedMeshlet {
                range: meshlet,
                diffuse,
                lightmap,
            });
        }

        let reference = context.graphics_context.mesh_cache.create_textured_mesh_reference(textured_mesh);
        context.asset_manager.model_cache.insert(hard_asset_key,reference.clone());
        Ok(reference)
    }
}

impl UserAssetMapping for TextAssetReference {
    type UserAsset = Rc<str>;

    async fn get_cached<IO: WimpyIO>(name: &'static str,context: &mut AssetLoadingContext<'_>) -> Result<Self::UserAsset,AssetManagerError> {
        let Some(virtual_asset) = context.asset_manager.manifest.text_assets.get(name) else {
            return Err(AssetManagerError::VirtualAssetNotFound(name));
        };
        Ok(context.asset_manager.get_text_cached::<IO>(virtual_asset.key,name).await?)
    }
}

impl UserAssetMapping for ImageAssetReference {
    type UserAsset = TextureFrameView;

    async fn get_cached<IO: WimpyIO>(name: &'static str,context: &mut AssetLoadingContext<'_>) -> Result<Self::UserAsset,AssetManagerError> {
        let Some(virtual_asset) = context.asset_manager.manifest.image_assets.get(name) else {
            return Err(AssetManagerError::VirtualAssetNotFound(name));
        };
        Ok(context.asset_manager.get_image_cached::<IO>(virtual_asset.key,name,virtual_asset.area,context.graphics_context).await?)
    }
}
