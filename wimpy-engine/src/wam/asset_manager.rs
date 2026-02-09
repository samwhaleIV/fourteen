use std::marker::PhantomData;

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

pub struct AssetManager<IO> {
    content_root: String,
    manifest: WamManifest,
    model_cache: ModelCache,
    phantom_data: PhantomData<IO>
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

struct Asset<'a,TData> {
    data: &'a TData,
    key: HardAssetKey,
    hard_asset: &'a HardAsset
}

impl<TData> Asset<'_,TData> {
    
}

impl<IO> AssetManager<IO>
where
    IO: WimpyIO
{
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
            phantom_data: Default::default(),
        }
    }

    fn try_get_asset<'a,TData>(&'a self,hard_asset_key: HardAssetKey,virtual_name: &str) -> Result<Asset<'a,TData>,AssetManagerError>
    where
        TData: DataResolver<TData>
    {
        let hard_asset = self.manifest.hard_assets.get(hard_asset_key).ok_or_else(
            || AssetManagerError::MissingHardAsset(virtual_name.to_string())
        )?;
        let data = TData::resolve_asset(hard_asset).ok_or_else(
            || AssetManagerError::MismatchedType { expected: TData::get_type(), found: hard_asset.data_type}
        )?;
        return Ok(Asset {
            data,
            hard_asset,
            key: hard_asset_key
        });
    }

    fn load_image(asset: Asset<'_, HardModelAsset>) {
        //todo make get or load interface
    }

    fn load_model(asset: Asset<'_, HardModelAsset>) {
        
    }

    pub async fn get_texture(
        &mut self,
        name: &str,
        graphics_context: &GraphicsContext
    ) -> Result<FrameCacheReference,AssetManagerError> {
        todo!();
    }

    pub async fn get_model(
        &mut self,
        name: &str,
        graphics_context: &GraphicsContext
    ) -> Result<ModelData<'_>,AssetManagerError> {

        let Some(virtual_asset) = self.manifest.virtual_assets.get(name) else {
            return Err(AssetManagerError::VirtualAssetNotFound(name.to_string()));
        };

        let v_data = match virtual_asset {
            VirtualAsset::Model(data) => data,
            _ => return Err(AssetManagerError::MismatchedType {
                expected: HardAssetType::Model,
                found: virtual_asset.get_type()
            }),
        };

        let model = v_data.model.map(|key|self.try_get_asset::<HardModelAsset>(key,name)).transpose()?;
        let diffuse = v_data.model.map(|key|self.try_get_asset::<HardImageAsset>(key,name)).transpose()?;
        let lightmap = v_data.model.map(|key|self.try_get_asset::<HardImageAsset>(key,name)).transpose()?;

        todo!();
    }
}
