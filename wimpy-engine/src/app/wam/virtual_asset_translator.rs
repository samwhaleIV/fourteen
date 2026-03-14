use super::prelude::*;

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
                file_source: Rc::from(hard_asset_input.source),
                data_type: hard_asset_input.r#type,
            });

            self.namespaces_ids.insert(id,key);
        }
        return Ok(());
    }

    pub fn load_untyped_assets(&mut self,assets: Vec<VirtualAssetInput>) -> Result<(),WamManifestError> {
        for asset in assets.into_iter() {
            let rc_name = self.manifest.get_virtual_asset_name(asset.name,self.namespace_name);
            let Some(key) = self.namespaces_ids.get(&asset.id) else {
                return Err(WamManifestError::MissingAsset(MissingAssetInfo {
                    name: rc_name,
                    id: asset.id
                }));
            };

            let hard_asset = self.manifest.hard_assets.get(*key).unwrap();
            match hard_asset.data_type {
                HardAssetType::Text => {
                    self.manifest.text_assets.insert(rc_name.clone(),TextAssetReference {
                        name: rc_name,
                        key: *key
                    });
                },
                HardAssetType::Image => {
                    self.manifest.image_assets.insert(rc_name.clone(),ImageAssetReference {
                        name: rc_name,
                        key: *key,
                        area: None
                    });
                },
                HardAssetType::Model => {
                    return Err(WamManifestError::UnexpectedType(UnexpectedTypeInfo {
                        name: rc_name,
                        id: asset.id,
                        found_type: hard_asset.data_type
                    }));
                }
            };
        }
        return Ok(())
    }

    pub fn load_images(&mut self,images: Vec<VirtualImageAssetInput>) -> Result<(),WamManifestError> {
        for image in images.into_iter() {
            let rc_name = self.manifest.get_virtual_asset_name(image.name,self.namespace_name);
            let Some(key) = self.namespaces_ids.get(&image.id) else {
                return Err(WamManifestError::MissingAsset(MissingAssetInfo {
                    name: rc_name,
                    id: image.id
                }));
            };
            let hard_asset = self.manifest.hard_assets.get(*key).unwrap();
            if hard_asset.data_type != HardAssetType::Image {
                return Err(WamManifestError::AssetTypeMismatch(TypeMismatchInfo {
                    name: rc_name,
                    id: image.id,
                    expected_type: HardAssetType::Image,
                    found_type: hard_asset.data_type
                }));
            }
            self.manifest.image_assets.insert(rc_name.clone(),ImageAssetReference {
                name: rc_name,
                key: *key,
                area: Some(image.area)
            });
        }
        return Ok(());
    }

    pub fn load_models(&mut self,models: Vec<VirtualModelAssetInput>) -> Result<(),WamManifestError> {
        for model in models.into_iter() {
            let rc_name = self.manifest.get_virtual_asset_name(model.name,self.namespace_name);

            let Some(key) = self.namespaces_ids.get(&model.id) else {
                return Err(WamManifestError::MissingAsset(MissingAssetInfo {
                    name: rc_name,
                    id: model.id
                }));
            };

            let hard_asset = self.manifest.hard_assets.get(*key).unwrap();
            if hard_asset.data_type != HardAssetType::Model {
                return Err(WamManifestError::AssetTypeMismatch(TypeMismatchInfo {
                    name: rc_name,
                    id: model.id,
                    expected_type: HardAssetType::Model,
                    found_type: hard_asset.data_type
                }));
            }

            let mut meshlets: Vec<MeshletDescriptor> = Vec::with_capacity(model.meshlets.len());

            for meshlet in &model.meshlets {
                let mut ref_meshlet = MeshletDescriptor {
                    diffuse: None,
                    lightmap: None,
                };
                let fields = [
                    (meshlet.diffuse_id,MeshletField::Diffuse),
                    (meshlet.lightmap_id,MeshletField::Lightmap)
                ];
                for (id,field) in fields {
                    let Some(id) = id else {
                        continue;
                    };
                    let Some(key) = self.namespaces_ids.get(&id) else {
                        return Err(WamManifestError::MissingAsset(MissingAssetInfo {
                            name: rc_name,
                            id
                        }));
                    };
                    let hard_asset = self.manifest.hard_assets.get(*key).unwrap();
                    if hard_asset.data_type != HardAssetType::Image {
                        return Err(WamManifestError::MismatchedMeshletField(MismatchedMeshletFieldInfo {
                            name: rc_name,
                            field,
                            expected_type: HardAssetType::Image,
                            found_type: hard_asset.data_type
                        }));
                    }
                    match field {
                        MeshletField::Diffuse => ref_meshlet.diffuse = Some(*key),
                        MeshletField::Lightmap => ref_meshlet.lightmap = Some(*key),
                    }
                }
                meshlets.push(ref_meshlet);
            }

            self.manifest.model_assets.insert(rc_name.clone(),ModelAssetReference {
                name: rc_name,
                key: *key,
                meshlet_descriptors: meshlets
            });
        }

        return Ok(());
    }
}
