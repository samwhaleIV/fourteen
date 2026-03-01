use super::prelude::*;
use crate::app::{AssetLoadingContext, FileError};

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

pub struct ImageSliceData {
    pub texture: TextureFrame,
    pub area: ImageArea
}

#[derive(Default,Debug)]
pub struct ModelData {
    pub cache_reference: Option<ModelCacheReference>,
    pub diffuse: Option<TextureFrame>,
    pub lightmap: Option<TextureFrame>,
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

    fn get_hard_asset_container<'a,TData>(hard_assets: &'a mut SlotMap<HardAssetKey, HardAsset>,hard_asset_key: HardAssetKey,name: &Rc<str>) -> Result<(&'a mut TData,Rc<str>),AssetManagerError>
    where
        TData: HardAssetResolver
    {
        let hard_asset = hard_assets.get_mut(hard_asset_key).ok_or_else(||AssetManagerError::MissingHardAsset(name.clone()))?;
        let data_type = hard_asset.data_type;
        let file_source = hard_asset.file_source.clone();
        let data = TData::resolve_to_type(hard_asset).ok_or_else(||AssetManagerError::MismatchedType {
            expected: TData::get_type(),
            found: data_type
        })?;
        return Ok((data,file_source));
    }

    async fn get_text<IO: WimpyIO>(&mut self,key: HardAssetKey,name: &Rc<str>) -> Result<Rc<str>,AssetManagerError> {
        let text = Self::get_hard_asset_container::<HardTextAsset>(&mut self.manifest.hard_assets,key,name)?;

        match &text.0.state {
            HardAssetState::Unloaded => {
                let path = get_full_path(&self.root,text.1.as_ref());
                let text_data: Rc<str> = Rc::from(match IO::load_text_file(path.as_path()).await {
                    Ok(data) => data,
                    Err(error) => return Err(AssetManagerError::FileError(error)),
                });
                text.0.state = HardAssetState::Loaded(text_data.clone());
                Ok(text_data)
            },
            HardAssetState::Loaded(text) => Ok(text.clone()),
        }
    }

    async fn get_image<IO: WimpyIO>(&mut self,key: HardAssetKey,name: &Rc<str>,graphics_context: &mut GraphicsContext) -> Result<TextureFrame,AssetManagerError> {
        let image = Self::get_hard_asset_container::<HardImageAsset>(&mut self.manifest.hard_assets,key,name)?;

        match image.0.state {
            HardAssetState::Unloaded => {
                let path = get_full_path(&self.root,image.1.as_ref());
                let image_data = match IO::load_image_file(path.as_path()).await {
                    Ok(data) => data,
                    Err(error) => return Err(AssetManagerError::FileError(error)),
                };
                let texture_frame = match graphics_context.create_texture_frame(image_data) {
                    Ok(value) => value,
                    Err(error) => return Err(AssetManagerError::TextureImportError(error)),
                };
                image.0.state = HardAssetState::Loaded(texture_frame);
                Ok(texture_frame)
            },
            HardAssetState::Loaded(cache_ref) => Ok(cache_ref),
        }
    }

    async fn get_mesh<IO: WimpyIO>(&mut self,key: HardAssetKey,name: &Rc<str>,graphics_context: &mut GraphicsContext) -> Result<ModelCacheReference,AssetManagerError> {
        let model = Self::get_hard_asset_container::<HardModelAsset>(&mut self.manifest.hard_assets,key,name)?;

        match model.0.state {
            HardAssetState::Unloaded => {
                let path = get_full_path(&self.root,model.1.as_ref());
                let gltf_data = match IO::load_binary_file(path.as_path()).await {
                    Ok(data) => data,
                    Err(error) => return Err(AssetManagerError::FileError(error)),
                };
                match graphics_context.create_model_cache_entry(&gltf_data) {
                    Ok(value) => {
                        model.0.state = HardAssetState::Loaded(value);
                        Ok(value)
                    },
                    Err(error) => Err(AssetManagerError::ModelImportError(error)),
                }
            },
            HardAssetState::Loaded(cache_ref) => Ok(cache_ref),
        }
    }
}

pub mod generic_types {
    pub struct Image;
    pub struct ImageSlice;
    pub struct Model;
    pub struct Text;
}

pub trait UserAssetMapping:  {
    type VirtualReference: AssetReferenceResolver + Clone;
    type UserAsset;
    fn get_user_asset<IO: WimpyIO>(asset: Self::VirtualReference,context: &mut AssetLoadingContext<'_>) -> impl Future<Output = Result<Self::UserAsset,AssetManagerError>>;
}

impl UserAssetMapping for generic_types::Model {

    type VirtualReference = ModelAssetReference;
    type UserAsset = ModelData;

    async fn get_user_asset<IO: WimpyIO>(asset: Self::VirtualReference,context: &mut AssetLoadingContext<'_>) -> Result<Self::UserAsset,AssetManagerError> {
        let diffuse = match asset.diffuse {
            Some(key) => Some(context.asset_manager.get_image::<IO>(key,&asset.name,context.graphics_context).await?),
            None => None,
        };

        let lightmap = match asset.lightmap {
            Some(key) => Some(context.asset_manager.get_image::<IO>(key,&asset.name,context.graphics_context).await?),
            None => None,
        };

        let mesh = match asset.model {
            Some(key) => Some(context.asset_manager.get_mesh::<IO>(key,&asset.name,context.graphics_context).await?),
            None => None,
        };

        Ok(ModelData {
            cache_reference: mesh,
            diffuse,
            lightmap
        })
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
        Ok(context.asset_manager.get_image::<IO>(asset.key,&asset.name,context.graphics_context).await?)
    }
}

impl UserAssetMapping for generic_types::ImageSlice {

    type VirtualReference = ImageSliceAssetReference;
    type UserAsset = ImageSliceData;

    async fn get_user_asset<IO: WimpyIO>(asset: Self::VirtualReference,context: &mut AssetLoadingContext<'_>) -> Result<Self::UserAsset,AssetManagerError> {
        Ok(ImageSliceData {
            texture: context.asset_manager.get_image::<IO>(asset.key,&asset.name,context.graphics_context).await?,
            area: asset.area
        })
    }
}
