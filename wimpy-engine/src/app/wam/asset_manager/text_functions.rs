use super::*;

impl AssetManager {
    pub fn get_text_reference(&self,name: &str) -> Result<TextAssetReference,AssetManagerError> {
        self.get_virtual_asset::<TextAssetReference>(&Rc::from(name))
    }

    pub async fn load_text_or_get_cached<IO: WimpyIO>(
        &mut self,
        key: HardAssetKey,
        name: &Rc<str>,
    ) -> Result<Rc<str>,AssetManagerError> {
        let text = Self::get_hard_asset_mut::<HardTextAsset>(&mut self.manifest.hard_assets,key,name)?;

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

    pub fn get_text(
        &self,
        reference: &TextAssetReference,
    ) -> Result<Rc<str>,AssetManagerError> {
        let text = self.get_cached_text(reference.key,&reference.name)?;
        return Ok(text);
    }

    pub async fn load_text<IO: WimpyIO>(
        &mut self,
        reference: &TextAssetReference,
    ) -> Result<Rc<str>,AssetManagerError> {
        let text = self.load_text_or_get_cached::<IO>(reference.key,&reference.name).await?;
        Ok(text)
    }

    pub fn get_cached_text(
        &self,
        key: HardAssetKey,
        name: &Rc<str>,
    ) -> Result<Rc<str>,AssetManagerError> {
        let text = Self::get_hard_asset::<HardTextAsset>(&self.manifest.hard_assets,key,name)?;

        match &text.data.state {
            HardAssetState::Unloaded => Err(AssetManagerError::AssetNotLoaded(name.clone())),
            HardAssetState::Loaded(text) => Ok(text.clone()),
        }
    }
}
