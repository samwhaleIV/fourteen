use std::collections::HashMap;
use crate::wam::*;

pub struct VirtualAssetTranslator<'a> {
    pub namespaces_ids: HashMap::<u32,HardAssetKey>,
    pub namespace_name: &'a str,
    pub manifest: &'a mut WamManifest
}

impl VirtualAssetTranslator<'_> {
    pub fn load_hard_assets(&mut self,hard_assets: Vec<HardAssetInput>) -> Result<(),WamManifestError> {
        for hard_asset_input in hard_assets.into_iter() {
            let id = hard_asset_input.id;

            let key = self.manifest.hard_assets.insert(HardAsset {
                file_source: hard_asset_input.source,
                data_type: hard_asset_input.r#type,
                data: HardAssetData::get_uninit(hard_asset_input.r#type),
            });

            self.namespaces_ids.insert(id,key);
        }
        return Ok(());
    }

    pub fn load_untyped_assets(&mut self,assets: Vec<VirtualAssetInput>) -> Result<(),WamManifestError> {
        for asset in assets.into_iter() {
            let Some(key) = self.namespaces_ids.get(&asset.id) else {
                return Err(WamManifestError::MissingAsset(MissingAssetInfo {
                    name: asset.name,
                    id: asset.id
                }));
            };
            let hard_asset = self.manifest.hard_assets.get(*key).unwrap();
            let value = match hard_asset.data_type {
                HardAssetType::Text => VirtualAsset::Text(*key),
                HardAssetType::Image => VirtualAsset::Image(*key),
                HardAssetType::Json => VirtualAsset::Json(*key),
                HardAssetType::Model => return Err(WamManifestError::UnexpectedType(UnexpectedTypeInfo {
                    name: asset.name,
                    id: asset.id,
                    found_type: hard_asset.data_type
                }))
            };
            self.manifest.add_virtual_asset(value,asset.name,self.namespace_name);
        }
        return Ok(())
    }

    pub fn load_images(&mut self,images: Vec<VirtualImageAssetInput>) -> Result<(),WamManifestError> {
        for image in images.into_iter() {
            let Some(key) = self.namespaces_ids.get(&image.id) else {
                return Err(WamManifestError::MissingAsset(MissingAssetInfo {
                    name: image.name,
                    id: image.id
                }));
            };
            let hard_asset = self.manifest.hard_assets.get(*key).unwrap();
            if hard_asset.data_type != HardAssetType::Image {
                return Err(WamManifestError::AssetTypeMismatch(TypeMismatchInfo {
                    name: image.name,
                    id: image.id,
                    expected_type: HardAssetType::Image,
                    found_type: hard_asset.data_type
                }));
            }
            self.manifest.add_virtual_asset(
                VirtualAsset::ImageSlice {
                    key: *key,
                    area: image.area
                },
                image.name,
                self.namespace_name
            );
        }
        return Ok(());
    }

    pub fn load_models(&mut self,models: Vec<VirtualModelAsset>) -> Result<(),WamManifestError> {
        for model in models.into_iter() {
            let assets = [
                (model.model_id,HardAssetType::Model,ModelField::Model),
                (model.diffuse_id,HardAssetType::Image,ModelField::Diffuse),
                (model.lightmap_id,HardAssetType::Image,ModelField::Lightmap),
            ];

            let mut model_data = VirtualModelData {
                model: None,
                diffuse: None,
                lightmap: None,
            };

            for (id,expected_type,field) in assets {
                let Some(id) = id else {
                    continue;
                };
                let Some(key) = self.namespaces_ids.get(&id) else {
                    return Err(WamManifestError::MissingAsset(MissingAssetInfo {
                        name: model.name,
                        id
                    }));
                };
                let hard_asset = self.manifest.hard_assets.get(*key).unwrap();
                if hard_asset.data_type != expected_type {
                    return Err(WamManifestError::MismatchedModelResource(MismatchedModelResourceInfo {
                        name: model.name,
                        field,
                        expected_type,
                        found_type: hard_asset.data_type
                    }));
                }

                match field {
                    ModelField::Model => model_data.model = Some(*key),
                    ModelField::Diffuse => model_data.diffuse = Some(*key),
                    ModelField::Lightmap => model_data.lightmap = Some(*key),
                }
            }

            self.manifest.add_virtual_asset(
                VirtualAsset::Model(model_data),
                model.name,
                self.namespace_name
            );
        }

        return Ok(());
    }
}
