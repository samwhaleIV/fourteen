use std::{marker::PhantomData, rc::Rc};

use slotmap::SlotMap;

use crate::{
    WimpyIO,
    wam::*,
    wgpu::{
        FrameCacheReference,
        GraphicsContext,
        GraphicsContextConfig,
        GraphicsProvider
    }
};

pub struct AssetManager {
    content_root: String,
    manifest: WamManifest,
    model_cache: ModelCache,
}

pub enum AssetManagerError {
    VirtualAssetNotFound(String),
    MismatchedType {
        expected: HardAssetType,
        found: HardAssetType
    },
    MissingHardAsset(String)
}

#[derive(Debug)]
pub struct ModelData<'a> {
    pub model: Option<RenderBufferReference>,
    pub collision: Option<&'a CollisionShape>,
    pub diffuse: Option<FrameCacheReference>,
    pub lightmap: Option<FrameCacheReference>,
}

pub struct ImageSliceData {
    pub texture_reference: FrameCacheReference,
    pub area: ImageArea
}

struct Asset<'a,TData> {
    data: &'a mut TData,
    file_source: Rc<str>
}

impl AssetManager {
    pub fn create<TConfig>(graphics_provider: &GraphicsProvider,content_root: String,manifest: WamManifest) -> Self
    where
        TConfig: GraphicsContextConfig
    {
        return Self {
            manifest,
            model_cache: ModelCache::create(
                graphics_provider,
                TConfig::MODEL_CACHE_VERTEX_BUFFER_SIZE,
                TConfig::MODEL_CACHE_INDEX_BUFFER_SIZE
            ),
            content_root,
        }
    }

    fn get_hard_asset<'a,TData>(hard_assets: &'a mut SlotMap<HardAssetKey,HardAsset>,hard_asset_key: HardAssetKey,virtual_name: &str) -> Result<Asset<'a,TData>,AssetManagerError>
    where
        TData: DataResolver<TData>
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

    pub async fn load_text(
        &mut self,
        asset: &VirtualTextAsset,
    ) -> Result<String,AssetManagerError> {
        todo!();
    }

    pub async fn load_image(
        &mut self,
        asset: &VirtualImageAsset,
        graphics_context: &GraphicsContext
    ) -> Result<FrameCacheReference,AssetManagerError> {
        todo!();
    }

    pub async fn load_image_slice_asset(
        &mut self,
        asset: &VirtualImageAsset,
        graphics_context: &GraphicsContext
    ) -> Result<ImageSliceData,AssetManagerError> {
        todo!();
    }

    pub async fn load_model(
        &mut self,
        asset: &VirtualModelAsset,
        graphics_context: &GraphicsContext
    ) -> Result<ModelData<'_>,AssetManagerError> {

        let mut hard_assets = &mut self.manifest.hard_assets;

        if let Some(model) = asset.model.map(|key|
            Self::get_hard_asset::<HardModelAsset>(hard_assets,key,&asset.name)
        ).transpose()? {

        }

        if let Some(diffuse) = asset.diffuse.map(|key|
            Self::get_hard_asset::<HardImageAsset>(hard_assets,key,&asset.name)
        ).transpose()? {

        }

        if let Some(lightmap) = asset.lightmap.map(|key|
            Self::get_hard_asset::<HardImageAsset>(hard_assets,key,&asset.name)
        ).transpose()? {

        }

        todo!();
    }
}
