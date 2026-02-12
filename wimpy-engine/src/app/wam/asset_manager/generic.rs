use super::*;

pub struct Asset<'a,TData> {
    pub data: &'a TData,
    pub file_source: Rc<str>
}

pub struct AssetMut<'a,TData> {
    pub data: &'a mut TData,
    pub file_source: Rc<str>
}

impl AssetManager {
    pub fn get_virtual_asset<TVirtualAsset>(&self,name: &Rc<str>) -> Result<TVirtualAsset,AssetManagerError>
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
            None => return Err(AssetManagerError::VirtualAssetNotFound(name.clone()))
        };
    }

    pub fn get_hard_asset<'a,TData>(
        hard_assets: &'a SlotMap<HardAssetKey,HardAsset>,
        hard_asset_key: HardAssetKey,
        name: &Rc<str>
    ) -> Result<Asset<'a,TData>,AssetManagerError>
    where
        TData: HardAssetResolver
    {
        let hard_asset = hard_assets.get(hard_asset_key).ok_or_else(
            || AssetManagerError::MissingHardAsset(name.clone())
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

    pub fn get_hard_asset_mut<'a,TData>(
        hard_assets: &'a mut SlotMap<HardAssetKey,HardAsset>,
        hard_asset_key: HardAssetKey,
        name: &Rc<str>
    
    ) -> Result<AssetMut<'a,TData>,AssetManagerError>
    where
        TData: HardAssetResolver
    {
        let hard_asset = hard_assets.get_mut(hard_asset_key).ok_or_else(
            || AssetManagerError::MissingHardAsset(name.clone())
        )?;
        let data_type = hard_asset.data_type;
        let file_source = hard_asset.file_source.clone();
        let data = TData::type_check_mut(hard_asset).ok_or_else(
            || AssetManagerError::MismatchedType { expected: TData::get_type(), found: data_type}
        )?;
        return Ok(AssetMut {
            data,
            file_source
        });
    }
}
