use std::{
    path::PathBuf,
    rc::Rc
};

const PATH_BUFFER_START_SIZE: usize = 64;

use slotmap::SlotMap;

use crate::{
    WimpyFileError,
    WimpyIO,
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
    pub meshes: Meshes<'a>,
    pub diffuse_texture: Option<TextureFrame>,
    pub lightmap_texture: Option<TextureFrame>,
}

pub struct ImageSliceData {
    pub texture: TextureFrame,
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
        TData: HardAssetResolver
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
        TVirtualAsset: AssetReferenceResolver<TVirtualAsset> + Clone
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

    async fn load_model_or_get_cached<IO: WimpyIO>(
        &mut self,
        key: HardAssetKey,
        name: &str,
        graphics_context: &GraphicsContext,
    ) -> Result<ModelCacheReference,AssetManagerError> {
        let model = Self::get_hard_asset::<HardModelAsset>(&mut self.manifest.hard_assets,key,name)?;

        match model.data.state {
            HardAssetState::Unloaded => {
                self.path_buffer.push(model.file_source.as_ref());
                let gltf_data = match IO::load_binary_file(self.path_buffer.as_path()).await {
                    Ok(data) => data,
                    Err(error) => return Err(AssetManagerError::FileError(error)),
                };
                self.path_buffer.pop();
                match self.model_cache.create_entry(graphics_context.get_graphics_provider(),&gltf_data) {
                    Ok(value) => {
                        model.data.state = HardAssetState::Loaded(value);
                        Ok(value)
                    },
                    Err(error) => Err(AssetManagerError::ModelImportError(error)),
                }
            },
            HardAssetState::Loaded(cache_ref) => Ok(cache_ref),
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
            HardAssetState::Unloaded => {
                self.path_buffer.push(image.file_source.as_ref());
                let image_data = match IO::load_image_file(self.path_buffer.as_path()).await {
                    Ok(data) => data,
                    Err(error) => return Err(AssetManagerError::FileError(error)),
                };
                self.path_buffer.pop();
                let texture_frame = match graphics_context.create_texture_frame(&image_data) {
                    Ok(value) => value,
                    Err(error) => return Err(AssetManagerError::TextureImportError(error)),
                };
                image.data.state = HardAssetState::Loaded(texture_frame);
                Ok(texture_frame)
            },
            HardAssetState::Loaded(cache_ref) => Ok(cache_ref),
        }
    }

    async fn load_text_or_get_cached<IO: WimpyIO>(
        &mut self,
        key: HardAssetKey,
        name: &str,
    ) -> Result<Rc<str>,AssetManagerError> {
        let text = Self::get_hard_asset::<HardTextAsset>(&mut self.manifest.hard_assets,key,name)?;

        match &text.data.state {
            HardAssetState::Unloaded => {
                self.path_buffer.push(text.file_source.as_ref());
                let text_data: Rc<str> = Rc::from(match IO::load_text_file(self.path_buffer.as_path()).await {
                    Ok(data) => data,
                    Err(error) => return Err(AssetManagerError::FileError(error)),
                });
                self.path_buffer.pop();
                text.data.state = HardAssetState::Loaded(text_data.clone());
                Ok(text_data)
            },
            HardAssetState::Loaded(text) => Ok(text.clone()),
        }
    }

    pub fn get_text_reference(&self,name: &str) -> Result<TextAssetReference,AssetManagerError> {
        self.get_virtual_asset::<TextAssetReference>(name)
    }

    pub fn get_image_reference(&self,name: &str) -> Result<ImageAssetReference,AssetManagerError> {
        self.get_virtual_asset::<ImageAssetReference>(name)
    }

    pub fn get_image_slice_reference(&self,name: &str) -> Result<VirtualImageSliceAsset,AssetManagerError> {
        self.get_virtual_asset::<VirtualImageSliceAsset>(name)
    }

    pub fn get_model_reference(&self,name: &str) -> Result<ModelAssetReference,AssetManagerError> {
        self.get_virtual_asset::<ModelAssetReference>(name)
    }

    pub async fn get_text<IO: WimpyIO>(
        &mut self,
        reference: &TextAssetReference,
    ) -> Result<Rc<str>,AssetManagerError> {
        let text = self.load_text_or_get_cached::<IO>(reference.key,&reference.name).await?;
        return Ok(text);
    }

    pub async fn get_image<IO: WimpyIO>(
        &mut self,
        reference: &ImageAssetReference,
        graphics_context: &mut GraphicsContext
    ) -> Result<TextureFrame,AssetManagerError> {
        let texture = self.load_image_or_get_cached::<IO>(reference.key,&reference.name,graphics_context).await?;
        return Ok(texture);
    }

    pub async fn get_image_slice<IO: WimpyIO>(
        &mut self,
        reference: &VirtualImageSliceAsset,
        graphics_context: &mut GraphicsContext
    ) -> Result<ImageSliceData,AssetManagerError> {
        let texture = self.load_image_or_get_cached::<IO>(reference.key,&reference.name,graphics_context).await?;
        return Ok(ImageSliceData {
            texture: texture,
            area: reference.area,
        });
    }

    pub async fn get_model<IO: WimpyIO>(
        &mut self,
        asset: &ModelAssetReference,
        graphics_context: &mut GraphicsContext,
    ) -> Result<ModelData<'_>,AssetManagerError> {

        let diffuse = match asset.diffuse {
            Some(key) => Some(self.load_image_or_get_cached::<IO>(key,&asset.name,graphics_context).await?),
            None => None,
        };

        let lightmap = match asset.lightmap {
            Some(key) => Some(self.load_image_or_get_cached::<IO>(key,&asset.name,graphics_context).await?),
            None => None,
        };

        let meshes = match asset.model {
            Some(key) => {
                let model_cache_reference = self.load_model_or_get_cached::<IO>(key,&asset.name,graphics_context).await?;
                self.model_cache.get_meshes(model_cache_reference)
            },
            None => Default::default(),
        };

        return Ok(ModelData {
            meshes,
            diffuse_texture: diffuse,
            lightmap_texture: lightmap,
        })
    }
}
