use slotmap::SparseSecondaryMap;

use super::prelude::*;
use crate::app::{AssetLoadingContext, FileError};

use crate::app::graphics::{GraphicsContext, ModelError, TextureError, TextureFrame, TexturedMeshReference, TexturedMeshlet};

const START_CACHE_ENTRY_CAPACITY: usize = 8;

#[derive(Default)]
pub struct AssetManager {
    manifest: WamManifest,
    root: PathBuf,
    // texture_frame_cache: SlotMap<ImageCacheKey,CacheState<TextureFrame>>,
    text_cache: SparseSecondaryMap<HardAssetKey,Rc<str>>,
    image_cache: SparseSecondaryMap<HardAssetKey,TextureFrame>,
    model_cache: SparseSecondaryMap<HardAssetKey,TextAssetReference>,
}

pub struct ImageSliceData {
    pub texture: TextureFrame,
    pub area: ImageArea
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
        };
    }

    pub fn get_virtual_asset<TReferenceResolver>(&self,name: &Rc<str>) -> Result<TReferenceResolver,AssetManagerError>
    where
        TReferenceResolver: AssetReferenceResolver + Clone
    {
        return match self.manifest.virtual_assets.get(name) {
            Some(virtual_asset) => match TReferenceResolver::type_check(virtual_asset) {
                Some(typed_virtual_asset) => Ok(typed_virtual_asset.clone()),
                None => return Err(AssetManagerError::MismatchedType {
                    expected: HardAssetType::Text,
                    found: virtual_asset.get_type()
                })
            },
            None => return Err(AssetManagerError::VirtualAssetNotFound(name.clone()))
        };
    }

    async fn get_text<IO: WimpyIO>(&mut self,key: HardAssetKey,name: &Rc<str>) -> Result<Rc<str>,AssetManagerError> {
        if let Some(text) = self.text_cache.get(key) {
            return Ok(text.clone());
        }
        let hard_asset = match self.manifest.hard_assets.get(key) {
            Some(value) => value,
            None => return Err(AssetManagerError::MissingHardAsset(name.clone())),
        };

        if hard_asset.data_type != HardAssetType::Text {
            return Err(AssetManagerError::MismatchedType {
                expected: HardAssetType::Text,
                found: hard_asset.data_type 
            })
        }

        let path = get_full_path(&self.root,&hard_asset.file_source);
        let text_data: Rc<str> = Rc::from(match IO::load_text_file(path.as_path()).await {
            Ok(data) => data,
            Err(error) => return Err(AssetManagerError::FileError(error)),
        });

        self.text_cache.insert(key,text_data.clone());

        Ok(text_data)
    }

    async fn get_image_cached<IO: WimpyIO>(&mut self,key: HardAssetKey,name: &Rc<str>,graphics_context: &mut GraphicsContext) -> Result<TextureFrame,AssetManagerError> {

        match image.0.state {
            AssetCacheState::Unloaded => {
                let path = get_full_path(&self.root,image.1.as_ref());
                let image_data = match IO::load_image_file(path.as_path()).await {
                    Ok(data) => data,
                    Err(error) => return Err(AssetManagerError::FileError(error)),
                };
                let texture_frame = match graphics_context.create_texture_frame(image_data) {
                    Ok(value) => value,
                    Err(error) => return Err(AssetManagerError::TextureImportError(error)),
                };
                image.0.state = AssetCacheState::Loaded(texture_frame);
                Ok(texture_frame)
            },
            AssetCacheState::Loaded(cache_ref) => Ok(cache_ref),
        }
    }

    // async fn get_image_anonymous<IO: WimpyIO>(&mut self,key: HardAssetKey,name: &Rc<str>,graphics_context: &mut GraphicsContext) -> Result<TextureFrame,AssetManagerError> {
    //     let image = Self::get_hard_asset_container::<HardImageAsset>(&mut self.manifest.hard_assets,key,name)?;

    //     match image.0.state {
    //         HardAssetState::Unloaded => {
    //             let path = get_full_path(&self.root,image.1.as_ref());
    //             let image_data = match IO::load_image_file(path.as_path()).await {
    //                 Ok(data) => data,
    //                 Err(error) => return Err(AssetManagerError::FileError(error)),
    //             };
    //             let texture_frame = match graphics_context.create_texture_frame(image_data) {
    //                 Ok(value) => value,
    //                 Err(error) => return Err(AssetManagerError::TextureImportError(error)),
    //             };
    //             image.0.state = HardAssetState::Loaded(texture_frame);
    //             Ok(texture_frame)
    //         },
    //         HardAssetState::Loaded(cache_ref) => Ok(cache_ref),
    //     }
    // }
}

pub mod generic_types {
    pub struct Image;
    pub struct ImageSlice;
    pub struct Model;
    pub struct Text;
}

pub trait UserAssetMapping {
    type VirtualReference: AssetReferenceResolver;
    type UserAsset;
    fn get_user_asset<IO: WimpyIO>(asset: Self::VirtualReference,context: &mut AssetLoadingContext<'_>) -> impl Future<Output = Result<Self::UserAsset,AssetManagerError>>;
}

impl UserAssetMapping for generic_types::Model {

    type VirtualReference = ModelAssetReference;
    type UserAsset = TexturedMeshReference;

    async fn get_user_asset<IO: WimpyIO>(asset: Self::VirtualReference,context: &mut AssetLoadingContext<'_>) -> Result<Self::UserAsset,AssetManagerError> {
        // A .gltf file
        let model = AssetManager::get_hard_asset_container::<HardModelAsset>(&mut context.asset_manager.manifest.hard_assets,asset.key,&asset.name)?;

        if let HardAssetState::Loaded(meshlets) = model.0.state {
            return Ok(meshlets);
        };

        let path = get_full_path(&context.asset_manager.root,model.1.as_ref());
        let gltf_data = match IO::load_binary_file(path.as_path()).await {
            Ok(data) => data,
            Err(error) => return Err(AssetManagerError::FileError(error)),
        };

        let queue = context.graphics_context.get_graphics_provider().get_queue();
        let (meshlet_count,mesh) = match context.mesh_cache.insert_geometry(queue,&gltf_data) {
            Ok(value) => value,
            Err(error) => return Err(AssetManagerError::ModelImportError(error)),
        };

        // There may be more meshlet descriptions than meshlet geometry, or vice versa
        let limit = meshlet_count.min(asset.meshlet_descriptors.len());

        let mut textured_mesh: Vec<TexturedMeshlet> = Vec::with_capacity(limit);

        for (i,meshlet) in mesh.enumerate() {
            let descriptor = &asset.meshlet_descriptors[i];

            let diffuse = match descriptor.diffuse {
                Some(key) => context.asset_manager.get_image_anonymous::<IO>(key,&asset.name,context.graphics_context).await?,
                None => context.graphics_context.engine_textures.missing,
            };

            let lightmap = match descriptor.lightmap {
                Some(key) => context.asset_manager.get_image_anonymous::<IO>(key,&asset.name,context.graphics_context).await?,
                None => context.graphics_context.engine_textures.opaque_white,
            };

            textured_mesh.push(TexturedMeshlet {
                meshlet,
                diffuse,
                lightmap,
            });
        }

        let reference = context.mesh_cache.create_textured_mesh_reference(textured_mesh);
        model.0.state = HardAssetState::Loaded(reference);
        Ok(reference)
    }
}

impl UserAssetMapping for generic_types::Text {

    type VirtualReference = TextAssetReference;
    type UserAsset = Rc<str>;

    async fn get_user_asset<IO: WimpyIO>(asset: Self::VirtualReference,context: &mut AssetLoadingContext<'_>) -> Result<Self::UserAsset,AssetManagerError> {
        Ok(context.asset_manager.get_text::<IO>(asset.key,&asset.name).await?)
    }
}

impl UserAssetMapping for generic_types::Image {

    type VirtualReference = ImageAssetReference;
    type UserAsset = TextureFrame;

    async fn get_user_asset<IO: WimpyIO>(asset: Self::VirtualReference,context: &mut AssetLoadingContext<'_>) -> Result<Self::UserAsset,AssetManagerError> {
        Ok(context.asset_manager.get_image_cached::<IO>(asset.key,&asset.name,context.graphics_context).await?)
    }
}

impl UserAssetMapping for generic_types::ImageSlice {

    type VirtualReference = ImageSliceAssetReference;
    type UserAsset = ImageSliceData;

    async fn get_user_asset<IO: WimpyIO>(asset: Self::VirtualReference,context: &mut AssetLoadingContext<'_>) -> Result<Self::UserAsset,AssetManagerError> {
        Ok(ImageSliceData {
            texture: context.asset_manager.get_image_cached::<IO>(asset.key,&asset.name,context.graphics_context).await?,
            area: asset.area
        })
    }
}
