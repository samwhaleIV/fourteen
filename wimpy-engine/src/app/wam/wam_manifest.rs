use std::{collections::HashMap, rc::Rc};
use slotmap::{SparseSecondaryMap, SlotMap};

use crate::UWimpyPoint;
use super::{HardAsset, json_input, HardAssetKey, HardAssetType, reference_types};

const DEFAULT_NAME_STRING_BUILDER_CAPACITY: usize = 64;
const DEFAULT_HARD_ASSET_CAPACITY: usize = 32;
const DEFAULT_VIRTUAL_ASSET_BUCKET_CAPACITY: usize = 32;

#[derive(Debug,Default)]
pub struct WamManifest {
    pub hard_assets: SlotMap<HardAssetKey,HardAsset>,

    pub size_hints: SparseSecondaryMap<HardAssetKey,UWimpyPoint>,

    pub text_assets:    HashMap<Rc<str>,    reference_types::Text>,
    pub image_assets:   HashMap<Rc<str>,    reference_types::Image>,
    pub model_assets:   HashMap<Rc<str>,    reference_types::Model>,

    string_builder: String,
}

#[derive(Debug)]
pub enum WamManifestError {
    MissingAsset {
        name: Rc<str>,
        id: u32
    },
    ImageMissingSizeHint {
        name: Rc<str>,
        id: u32
    },
    ImageSizeHintMissingOwner {
        id: u32
    },
    UnexpectedType{
        name: Rc<str>,
        id: u32,
        found_type: HardAssetType
    },
    AssetTypeMismatch{
        name: Rc<str>,
        id: u32,
        expected_type: HardAssetType,
        found_type: HardAssetType
    },
    MismatchedMeshletField{
        name: Rc<str>,
        field: reference_types::MeshletField,
        expected_type: HardAssetType,
        found_type: HardAssetType
    },
    IOError(std::io::Error),
    JsonError(String),
}

impl WamManifest {

    pub fn create(json_text: &str) -> Result<Self,WamManifestError> {

        let namespace_table: HashMap<String,json_input::Namespace> = match serde_json::from_str(&json_text) {
            Ok(value) => value,
            Err(error) => {
                // TODO: match the serde_json error instead of formatting it
                return Err(WamManifestError::JsonError(format!("{:?}",error)))
            },
        };

        let mut manifest = Self {
            hard_assets:    SlotMap::with_capacity_and_key      (DEFAULT_HARD_ASSET_CAPACITY),
            string_builder: String::with_capacity               (DEFAULT_NAME_STRING_BUILDER_CAPACITY),
            text_assets:    HashMap::with_capacity              (DEFAULT_VIRTUAL_ASSET_BUCKET_CAPACITY),
            image_assets:   HashMap::with_capacity              (DEFAULT_VIRTUAL_ASSET_BUCKET_CAPACITY),
            model_assets:   HashMap::with_capacity              (DEFAULT_VIRTUAL_ASSET_BUCKET_CAPACITY),
            size_hints:     SparseSecondaryMap::with_capacity   (DEFAULT_VIRTUAL_ASSET_BUCKET_CAPACITY),
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
        self.string_builder.insert_str(0,namespace_name);
        self.string_builder.push('/');
        self.string_builder.push_str(&local_name);
        local_name.clear();
        local_name.push_str(&self.string_builder);
        self.string_builder.clear();
        return Rc::from(local_name);
    }

    fn add_namespace(&mut self,namespace: json_input::Namespace,namespace_name: &str) -> Result<(),WamManifestError> {
        let hard_asset_count = namespace.hard_assets.len();

        /*  
            The WAM format does not specify if IDs are unique to a namespace or interned across instances.
            So, we sandbox namespaces and translate their IDs to runtime-only slotmap keys.
        */

        let mut translator = super::virtual_asset_translator::VirtualAssetTranslator {
            manifest: self,
            namespaces_ids: HashMap::with_capacity(hard_asset_count),
            namespace_name,
        };

        translator.parse_hard_assets    (namespace.hard_assets)?;
        translator.parse_size_hints     (namespace.image_size_hints);
        translator.parse_generic_assets (namespace.virtual_assets)?;
        translator.parse_slice_images   (namespace.virtual_image_slice_assets)?;
        translator.parse_models         (namespace.virtual_model_assets)?;

        return Ok(());
    }
}
