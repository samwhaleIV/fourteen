use super::*;

pub struct ImageSliceData {
    pub texture: TextureFrame,
    pub area: ImageArea
}

impl AssetManager {
    pub fn get_image_reference(&self,name: &str) -> Result<ImageAssetReference,AssetManagerError> {
        self.get_virtual_asset::<ImageAssetReference>(&Rc::from(name))
    }

    pub fn get_image_slice_reference(&self,name: &str) -> Result<VirtualImageSliceAsset,AssetManagerError> {
        self.get_virtual_asset::<VirtualImageSliceAsset>(&Rc::from(name))
    }

    pub(super) fn get_cached_image(
        &self,
        key: HardAssetKey,
        name: &Rc<str>,
    ) -> Result<TextureFrame,AssetManagerError> {
        let image = Self::get_hard_asset::<HardImageAsset>(&self.manifest.hard_assets,key,name)?;

        match image.data.state {
            HardAssetState::Unloaded => Err(AssetManagerError::AssetNotLoaded(name.clone())),
            HardAssetState::Loaded(cache_ref) => Ok(cache_ref),
        }
    }

    pub async fn load_image_or_get_cached<IO: WimpyIO>(
        &mut self,
        key: HardAssetKey,
        name: &Rc<str>,
        graphics_context: &mut GraphicsContext,
    ) -> Result<TextureFrame,AssetManagerError> {
        let image = Self::get_hard_asset_mut::<HardImageAsset>(&mut self.manifest.hard_assets,key,name)?;

        match image.data.state {
            HardAssetState::Unloaded => {
                let path = get_full_path(&self.root,image.path.as_ref());
                let image_data = match IO::load_image_file(path.as_path()).await {
                    Ok(data) => data,
                    Err(error) => return Err(AssetManagerError::FileError(error)),
                };
                let texture_frame = match graphics_context.create_texture_frame(image_data) {
                    Ok(value) => value,
                    Err(error) => return Err(AssetManagerError::TextureImportError(error)),
                };
                image.data.state = HardAssetState::Loaded(texture_frame);
                Ok(texture_frame)
            },
            HardAssetState::Loaded(cache_ref) => Ok(cache_ref),
        }
    }

    pub async fn load_image<IO: WimpyIO>(
        &mut self,
        reference: &ImageAssetReference,
        graphics_context: &mut GraphicsContext
    ) -> Result<TextureFrame,AssetManagerError> {
        let texture = self.load_image_or_get_cached::<IO>(reference.key,&reference.name,graphics_context).await?;
        Ok(texture)
    }

    pub fn get_image(
        &self,
        reference: &ImageAssetReference,
    ) -> Result<TextureFrame,AssetManagerError> {
        let texture = self.get_cached_image(reference.key,&reference.name)?;
        return Ok(texture);
    }

    pub async fn load_image_slice<IO: WimpyIO>(
        &mut self,
        reference: &VirtualImageSliceAsset,
        graphics_context: &mut GraphicsContext
    ) -> Result<ImageSliceData,AssetManagerError> {
        let texture = self.load_image_or_get_cached::<IO>(reference.key,&reference.name,graphics_context).await?;
        Ok(ImageSliceData {
            texture: texture,
            area: reference.area,
        })
    }

    pub fn get_image_slice(
        &self,
        reference: &VirtualImageSliceAsset,
    ) -> Result<ImageSliceData,AssetManagerError> {
        let texture = self.get_cached_image(reference.key,&reference.name)?;
        return Ok(ImageSliceData {
            texture: texture,
            area: reference.area,
        });
    }
}
