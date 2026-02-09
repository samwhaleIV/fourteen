use std::rc::Rc;
use crate::wam::*;
use crate::WimpyIO;
use crate::wgpu::GraphicsContext;
use crate::wgpu::TextureFrame;

#[derive(Debug)]
pub struct ModelData<'a> {
    pub meshes: Meshes<'a>,
    pub diffuse_texture: Option<TextureFrame>,
    pub lightmap_texture: Option<TextureFrame>,
}

impl AssetManager {
    fn get_cached_model(
        &self,
        key: HardAssetKey,
        name: &Rc<str>,
    ) -> Result<ModelCacheReference,AssetManagerError> {
        let model = Self::get_hard_asset::<HardModelAsset>(&self.manifest.hard_assets,key,name)?;

        match model.data.state {
            HardAssetState::Unloaded => Err(AssetManagerError::AssetNotLoaded(name.clone())),
            HardAssetState::Loaded(cache_ref) => Ok(cache_ref),
        }
    }

    async fn load_model_or_get_cached<IO: WimpyIO>(
        &mut self,
        key: HardAssetKey,
        name: &Rc<str>,
        graphics_context: &GraphicsContext,
    ) -> Result<ModelCacheReference,AssetManagerError> {
        let model = Self::get_hard_asset_mut::<HardModelAsset>(&mut self.manifest.hard_assets,key,name)?;

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

    pub fn get_model_reference(&self,name: &str) -> Result<ModelAssetReference,AssetManagerError> {
        self.get_virtual_asset::<ModelAssetReference>(&Rc::from(name))
    }

    pub async fn load_model<IO: WimpyIO>(
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

        Ok(ModelData {
            meshes,
            diffuse_texture: diffuse,
            lightmap_texture: lightmap,
        })
    }

    pub fn get_model(
        &self,
        asset: &ModelAssetReference,
    ) -> Result<ModelData<'_>,AssetManagerError> {

        let diffuse = match asset.diffuse {
            Some(key) => Some(self.get_cached_image(key,&asset.name)?),
            None => None,
        };

        let lightmap = match asset.lightmap {
            Some(key) => Some(self.get_cached_image(key,&asset.name)?),
            None => None,
        };

        let meshes = match asset.model {
            Some(key) => {
                let model_cache_reference = self.get_cached_model(key,&asset.name)?;
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
