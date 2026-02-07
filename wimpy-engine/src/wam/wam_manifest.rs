use std::collections::HashMap;
use serde::Deserialize;
use slotmap::SlotMap;

const NAME_BUILDING_BUFFER_START_CAPACITY: usize = 64;

#[derive(Deserialize,Debug)]
#[serde(rename_all = "kebab-case")]
pub struct InputNamespace {
    pub hard_assets: Vec<HardAssetInput>,
    pub virtual_assets: Vec<VirtualAssetInput>,
    pub virtual_image_assets: Vec<VirtualImageAssetInput>,
    pub virtual_model_assets: Vec<VirtualModelAsset>
}

slotmap::new_key_type! {
    pub struct HardAssetKey;
}

#[derive(Debug)]
pub struct HardAsset {
    pub file_source: String,
    pub asset_type: HardAssetType
}

#[derive(Debug)]
pub enum VirtualAsset {
    Text(HardAssetKey),
    TextureData(HardAssetKey),
    Json(HardAssetKey),
    Image(HardAssetKey),
    ImageWithArea(HardAssetKey,ImageArea),
    Model(ModelData)
}

#[derive(Debug)]
pub struct ModelData {
    pub model_id: Option<HardAssetKey>,
    pub diffuse_id: Option<HardAssetKey>,
    pub lightmap_id: Option<HardAssetKey>,
    pub collision_id: Option<HardAssetKey>,
}

#[derive(Debug,Default)]
pub struct WamManifest {
    pub hard_assets: SlotMap<HardAssetKey,HardAsset>,
    pub virtual_assets: HashMap<String,VirtualAsset>,
    string_building_buffer: String,
}

impl WamManifest {

    pub fn create(json_text: &str) -> Result<Self,WamManifestError> {

        let namespace_table: HashMap<String,InputNamespace> = match serde_json::from_str(&json_text) {
            Ok(value) => value,
            Err(error) => {
                // TODO: match the serde_json error instead of formatting it
                return Err(WamManifestError::JsonError(format!("{:?}",error)))
            },
        };


        let mut manifest = Self {
            hard_assets: SlotMap::<HardAssetKey,HardAsset>::with_key(),
            virtual_assets: Default::default(),
            string_building_buffer: String::with_capacity(NAME_BUILDING_BUFFER_START_CAPACITY)
        };

        let item_count = namespace_table.len();

        let mut id_table = HashMap::with_capacity(item_count);
        let mut namespaces = Vec::with_capacity(item_count);

        for (name,value) in namespace_table.into_iter() {
            let id = namespaces.len();
            namespaces.push(manifest.add_namespace(value,&name)?);
            id_table.insert(name,id);
        }

        return Ok(manifest);
    }

    fn add_virtual_asset(&mut self,asset: VirtualAsset,mut local_name: String,namespace_name: &str) {
        self.string_building_buffer.insert_str(0,namespace_name);
        self.string_building_buffer.push('/');
        self.string_building_buffer.push_str(&local_name);
        local_name.clear();
        local_name.push_str(&self.string_building_buffer);
        self.string_building_buffer.clear();
        self.virtual_assets.insert(local_name,asset);
    }

    fn add_namespace(&mut self,namespace: InputNamespace,namespace_name: &str) -> Result<(),WamManifestError> {
        let hard_asset_count = namespace.hard_assets.len();

        /*  
            The WAM format does not specify if IDs are unique to a namespace or interned across instances.
            So, we sandbox namespaces and translate their IDs to runtime-only slotmap keys.
        */

        let mut namespaces_ids = HashMap::<u32,HardAssetKey>::with_capacity(hard_asset_count);

        for hard_asset_input in namespace.hard_assets.into_iter() {
            let id = hard_asset_input.id;

            let key = self.hard_assets.insert(HardAsset {
                file_source: hard_asset_input.source,
                asset_type: hard_asset_input.r#type
            });

            namespaces_ids.insert(id,key);
        }

        for asset in namespace.virtual_assets.into_iter() {
            let Some(key) = namespaces_ids.get(&asset.id) else {
                return Err(WamManifestError::MissingAsset(MissingAssetInfo {
                    name: asset.name,
                    id: asset.id
                }));
            };
            let hard_asset = self.hard_assets.get(*key).unwrap();
            let value = match hard_asset.asset_type {
                HardAssetType::Text => {
                    VirtualAsset::Text(*key)
                },
                HardAssetType::Image => {
                    VirtualAsset::Image(*key)
                },
                HardAssetType::Json => {
                    VirtualAsset::Json(*key)
                },
                _ => return Err(WamManifestError::UnexpectedType(UnexpectedTypeInfo {
                    name: asset.name,
                    id: asset.id,
                    found_type: hard_asset.asset_type
                }))
            };
            self.add_virtual_asset(value,asset.name,namespace_name);
        }

        for image in namespace.virtual_image_assets {
            let Some(key) = namespaces_ids.get(&image.id) else {
                return Err(WamManifestError::MissingAsset(MissingAssetInfo {
                    name: image.name,
                    id: image.id
                }));
            };
            let hard_asset = self.hard_assets.get(*key).unwrap();
            if hard_asset.asset_type != HardAssetType::Image {
                return Err(WamManifestError::AssetTypeMismatch(TypeMismatchInfo {
                    name: image.name,
                    id: image.id,
                    expected_type: HardAssetType::Image,
                    found_type: hard_asset.asset_type
                }));
            }
            self.add_virtual_asset(
                VirtualAsset::ImageWithArea(*key,image.area),
                image.name,
                namespace_name
            );
        }

        for model in namespace.virtual_model_assets {
            let assets = [
                (model.model_id,HardAssetType::Model,ModelField::Model),
                (model.collision_id,HardAssetType::Model,ModelField::Collision),
                (model.diffuse_id,HardAssetType::Image,ModelField::Diffuse),
                (model.lightmap_id,HardAssetType::Image,ModelField::Lightmap),
            ];

            let mut model_data = ModelData {
                model_id: None,
                diffuse_id: None,
                lightmap_id: None,
                collision_id: None,
            };

            for (id,expected_type,field) in assets {
                let Some(id) = id else {
                    continue;
                };
                let Some(key) = namespaces_ids.get(&id) else {
                    return Err(WamManifestError::MissingAsset(MissingAssetInfo {
                        name: model.name,
                        id
                    }));
                };
                let hard_asset = self.hard_assets.get(*key).unwrap();
                if hard_asset.asset_type != expected_type {
                    return Err(WamManifestError::MismatchedModelResource(MismatchedModelResourceInfo {
                        name: model.name,
                        field,
                        expected_type,
                        found_type: hard_asset.asset_type 
                    }));
                }

                match field {
                    ModelField::Model => model_data.model_id = Some(*key),
                    ModelField::Collision => model_data.collision_id = Some(*key),
                    ModelField::Diffuse => model_data.collision_id = Some(*key),
                    ModelField::Lightmap => model_data.collision_id = Some(*key),
                }
            }

            self.add_virtual_asset(
                VirtualAsset::Model(model_data),
                model.name,
                namespace_name
            );
        }

        return Ok(());
    }
}

#[derive(Deserialize,Debug)]
pub struct HardAssetInput {
    pub id: u32,
    pub source: String,
    pub r#type: HardAssetType
}

#[derive(Deserialize,Debug,Copy,Clone,PartialEq,Eq)]
#[serde(rename_all = "lowercase")]
pub enum HardAssetType {
    Text,
    Image,
    Model,
    Json
}

#[derive(Deserialize,Debug)]
pub struct VirtualAssetInput {
    pub id: u32,
    pub name: String,
}

#[derive(Deserialize,Debug)]
pub struct VirtualImageAssetInput {
    pub id: u32,
    pub name: String,
    pub area: ImageArea
}

#[derive(Deserialize,Debug)]
pub struct ImageArea {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32
}

#[derive(Deserialize,Debug)]
#[serde(rename_all = "kebab-case")]
pub struct VirtualModelAsset {
    pub name: String,
    pub model_id: Option<u32>,
    pub diffuse_id: Option<u32>,
    pub lightmap_id: Option<u32>,
    pub collision_id: Option<u32>
}

#[derive(Debug)]
pub struct MissingAssetInfo {
    pub name: String,
    pub id: u32,
}

#[derive(Debug)]
pub struct TypeMismatchInfo {
    pub name: String,
    pub id: u32,
    pub expected_type: HardAssetType,
    pub found_type: HardAssetType
}

#[derive(Debug)]
pub struct UnexpectedTypeInfo {
    pub name: String,
    pub id: u32,
    pub found_type: HardAssetType
}

#[derive(Debug)]
pub enum ModelField {
    Model,
    Collision,
    Diffuse,
    Lightmap
}

#[derive(Debug)]
pub struct MismatchedModelResourceInfo {
    pub name: String,
    pub field: ModelField,
    pub expected_type: HardAssetType,
    pub found_type: HardAssetType
}

#[derive(Debug)]
pub enum WamManifestError {
    MissingAsset(MissingAssetInfo),
    UnexpectedType(UnexpectedTypeInfo),
    AssetTypeMismatch(TypeMismatchInfo),
    MismatchedModelResource(MismatchedModelResourceInfo),
    IOError(std::io::Error),
    JsonError(String)
}
