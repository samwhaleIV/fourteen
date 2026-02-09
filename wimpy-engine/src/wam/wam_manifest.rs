use std::{collections::HashMap, rc::Rc};
use serde::Deserialize;
use slotmap::SlotMap;

use crate::wam::*;

const NAME_BUILDING_BUFFER_START_CAPACITY: usize = 64;

#[derive(Deserialize,Debug)]
#[serde(rename_all = "kebab-case")]
pub struct InputNamespace {
    pub hard_assets: Vec<HardAssetInput>,
    pub virtual_assets: Vec<VirtualAssetInput>,
    pub virtual_image_assets: Vec<VirtualImageAssetInput>,
    pub virtual_model_assets: Vec<VirtualModelAssetInput>
}

slotmap::new_key_type! {
    pub struct HardAssetKey;
}

#[derive(Debug,Default)]
pub enum AssetState<T> {
    #[default]
    Unloaded,
    Loaded(T)
}

impl<T> AssetState<T> {
    pub fn is_loaded(&self) -> bool {
        return match self {
            AssetState::Unloaded => false,
            AssetState::Loaded(_) => true,
        }
    }
}

#[derive(Debug)]
pub struct HardAsset {
    pub file_source: Rc<str>,
    pub data_type: HardAssetType,
    pub data: HardAssetData,
}

#[derive(Debug,Default)]
pub struct WamManifest {
    pub hard_assets: SlotMap<HardAssetKey,HardAsset>,
    pub virtual_assets: HashMap<Rc<str>,VirtualAsset>,
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

    pub fn get_virtual_asset_name(&mut self,mut local_name: String,namespace_name: &str) -> Rc<str> {
        self.string_building_buffer.insert_str(0,namespace_name);
        self.string_building_buffer.push('/');
        self.string_building_buffer.push_str(&local_name);
        local_name.clear();
        local_name.push_str(&self.string_building_buffer);
        self.string_building_buffer.clear();
        return Rc::from(local_name);
    }

    pub fn add_virtual_asset(&mut self,asset: VirtualAsset,name: Rc<str>) {
        self.virtual_assets.insert(name,asset);
    }

    fn add_namespace(&mut self,namespace: InputNamespace,namespace_name: &str) -> Result<(),WamManifestError> {
        let hard_asset_count = namespace.hard_assets.len();

        /*  
            The WAM format does not specify if IDs are unique to a namespace or interned across instances.
            So, we sandbox namespaces and translate their IDs to runtime-only slotmap keys.
        */

        let mut translator = VirtualAssetTranslator {
            manifest: self,
            namespaces_ids: HashMap::with_capacity(hard_asset_count),
            namespace_name,
        };

        translator.load_hard_assets(namespace.hard_assets)?;
        translator.load_untyped_assets(namespace.virtual_assets)?;
        translator.load_images(namespace.virtual_image_assets)?;
        translator.load_models(namespace.virtual_model_assets)?;

        return Ok(());
    }
}

#[derive(Deserialize,Debug)]
pub struct HardAssetInput {
    pub id: u32,
    pub source: String,
    pub r#type: HardAssetType
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

#[derive(Deserialize,Debug,Copy,Clone)]
pub struct ImageArea {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32
}

#[derive(Deserialize,Debug)]
#[serde(rename_all = "kebab-case")]
pub struct VirtualModelAssetInput {
    pub name: String,
    pub model_id: Option<u32>,
    pub diffuse_id: Option<u32>,
    pub lightmap_id: Option<u32>,
}

#[derive(Debug)]
pub struct MissingAssetInfo {
    pub name: Rc<str>,
    pub id: u32,
}

#[derive(Debug)]
pub struct TypeMismatchInfo {
    pub name: Rc<str>,
    pub id: u32,
    pub expected_type: HardAssetType,
    pub found_type: HardAssetType
}

#[derive(Debug)]
pub struct UnexpectedTypeInfo {
    pub name: Rc<str>,
    pub id: u32,
    pub found_type: HardAssetType
}

#[derive(Debug)]
pub enum ModelField {
    Model,
    Diffuse,
    Lightmap
}

#[derive(Debug)]
pub struct MismatchedModelResourceInfo {
    pub name: Rc<str>,
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
