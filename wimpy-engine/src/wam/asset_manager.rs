use std::{
    path::{Path, PathBuf},
    rc::Rc
};

const PATH_BUFFER_START_SIZE: usize = 64;

use slotmap::SlotMap;

use crate::{
    WimpyFileError,
    WimpyIO,
    wam::*,
    wgpu::{
        GraphicsContext,
        GraphicsContextConfig,
        GraphicsContextController,
        GraphicsProvider,
        TextureContainerError,
        TextureFrame
    }
};

pub struct AssetManager {
    manifest: WamManifest,
    model_cache: ModelCache,
    path_buffer: PathBuf,
}

#[derive(Debug)]
pub enum AssetManagerError {
    VirtualAssetNotFound(String),
    MismatchedType {
        expected: HardAssetType,
        found: HardAssetType
    },
    MissingHardAsset(String),
    FileError(WimpyFileError),
    ModelImportError(ModelImportError),
    TextureImportError(TextureContainerError)
}

#[derive(Debug)]
pub struct ModelData<'a> {
    pub model: Option<RenderBufferReference>,
    pub collision: Option<&'a CollisionShape>,
    pub diffuse: Option<TextureFrame>,
    pub lightmap: Option<TextureFrame>,
}

pub struct ImageSliceData {
    pub texture_reference: TextureFrame,
    pub area: ImageArea
}

struct Asset<'a,TData> {
    data: &'a mut TData,
    file_source: Rc<str>
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

    fn get_hard_asset<'a,TData>(
        
        hard_assets: &'a mut SlotMap<HardAssetKey,HardAsset>,
        hard_asset_key: HardAssetKey,
        virtual_name: &str
    
    ) -> Result<Asset<'a,TData>,AssetManagerError>
    where
        TData: DataResolver
    {
        let hard_asset = hard_assets.get_mut(hard_asset_key).ok_or_else(
            || AssetManagerError::MissingHardAsset(virtual_name.to_string())
        )?;
        let data_type = hard_asset.data_type;
        let file_source = hard_asset.file_source.clone();
        let data = TData::type_check(hard_asset).ok_or_else(
            || AssetManagerError::MismatchedType { expected: TData::get_type(), found: data_type}
        )?;
        return Ok(Asset {
            data,
            file_source
        });
    }

    fn get_virtual_asset<TVirtualAsset>(&self,name: &str) -> Result<TVirtualAsset,AssetManagerError>
    where
        TVirtualAsset: VirtualAssetResolver<TVirtualAsset> + Clone
    {
        return match self.manifest.virtual_assets.get(name) {
            Some(virtual_asset) => match TVirtualAsset::type_check(virtual_asset) {
                Some(typed_virtual_asset) => Ok(typed_virtual_asset.clone()),
                None => return Err(AssetManagerError::MismatchedType {
                    expected: HardAssetType::Text,
                    found: virtual_asset.get_type()
                })
            },
            None => return Err(AssetManagerError::VirtualAssetNotFound(name.to_string()))
        };
    }

    pub fn find_text_asset(&self,name: &str) -> Result<VirtualTextAsset,AssetManagerError> {
        self.get_virtual_asset::<VirtualTextAsset>(name)
    }

    pub fn find_image_asset(&self,name: &str) -> Result<VirtualImageAsset,AssetManagerError> {
        self.get_virtual_asset::<VirtualImageAsset>(name)
    }

    pub fn find_image_slice_asset(&self,name: &str) -> Result<VirtualImageSliceAsset,AssetManagerError> {
        self.get_virtual_asset::<VirtualImageSliceAsset>(name)
    }

    pub fn find_model_asset(&self,name: &str) -> Result<VirtualModelAsset,AssetManagerError> {
        self.get_virtual_asset::<VirtualModelAsset>(name)
    }

    pub async fn load_text<IO: WimpyIO>(
        &mut self,
        asset: &VirtualTextAsset,
    ) -> Result<String,AssetManagerError> {
        todo!();
    }

    pub async fn load_image<IO: WimpyIO>(
        &mut self,
        asset: &VirtualImageAsset,
        graphics_context: &GraphicsContext
    ) -> Result<TextureFrame,AssetManagerError> {
        todo!();
    }

    pub async fn load_image_slice_asset<IO: WimpyIO>(
        &mut self,
        asset: &VirtualImageAsset,
        graphics_context: &GraphicsContext
    ) -> Result<ImageSliceData,AssetManagerError> {
        todo!();
    }

    async fn load_model_or_get_cached<IO: WimpyIO>(
        &mut self,
        key: HardAssetKey,
        name: &str,
        graphics_context: &GraphicsContext,
    ) -> Result<ModelCacheReference,AssetManagerError> {
        let model = Self::get_hard_asset::<HardModelAsset>(&mut self.manifest.hard_assets,key,name)?;
        match model.data.state {
            AssetState::Unloaded => {
                self.path_buffer.push(model.file_source.as_ref());
                let gltf_data = match IO::load_binary_file(self.path_buffer.as_path()).await {
                    Ok(data) => data,
                    Err(error) => return Err(AssetManagerError::FileError(error)),
                };
                self.path_buffer.pop();
                match self.model_cache.create_entry(graphics_context.get_graphics_provider(),&gltf_data) {
                    Ok(value) => {
                        model.data.state = AssetState::Loaded(value);
                        Ok(value)
                    },
                    Err(error) => Err(AssetManagerError::ModelImportError(error)),
                }
            },
            AssetState::Loaded(cache_ref) => Ok(cache_ref),
        }
    }

    async fn load_image_or_get_cached<IO: WimpyIO>(
        &mut self,
        key: HardAssetKey,
        name: &str,
        graphics_context: &mut GraphicsContext,
    ) -> Result<TextureFrame,AssetManagerError> {
        let image = Self::get_hard_asset::<HardImageAsset>(&mut self.manifest.hard_assets,key,name)?;

        match image.data.state {
            AssetState::Unloaded => {
                self.path_buffer.push(image.file_source.as_ref());
                let image_data = match IO::load_image(self.path_buffer.as_path()).await {
                    Ok(data) => data,
                    Err(error) => return Err(AssetManagerError::FileError(error)),
                };
                self.path_buffer.pop();
                let texture_frame = match graphics_context.create_texture_frame(&image_data) {
                    Ok(value) => value,
                    Err(error) => return Err(AssetManagerError::TextureImportError(error)),
                };
                image.data.state = AssetState::Loaded(texture_frame);
                Ok(texture_frame)
            },
            AssetState::Loaded(cache_ref) => Ok(cache_ref),
        }
    }

    pub async fn load_model<IO: WimpyIO>(

        &mut self,
        asset: &VirtualModelAsset,
        graphics_context: &mut GraphicsContext,

    ) -> Result<ModelData<'_>,AssetManagerError> {

        let mut model: Option<RenderBufferReference> = None;
        let mut collision: Option<&CollisionShape> = None;

        if let Some(key) = asset.model {
            let model_cache_reference = self.load_model_or_get_cached::<IO>(key,&asset.name,graphics_context).await?;
            let meshes = self.model_cache.get_meshes(model_cache_reference);
        }


        let diffuse = match asset.diffuse {
            Some(key) => Some(self.load_image_or_get_cached::<IO>(key,&asset.name,graphics_context).await?),
            None => None,
        };

        let lightmap = match asset.lightmap {
            Some(key) => Some(self.load_image_or_get_cached::<IO>(key,&asset.name,graphics_context).await?),
            None => None,
        };



        return Ok(ModelData {
            model,
            collision,
            diffuse,
            lightmap,
        })
    }
}
