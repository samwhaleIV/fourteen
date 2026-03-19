use std::rc::Rc;
use crate::{UWimpyPoint, WimpyPointRect};

mod virtual_asset_translator; /* Private */

mod wam_manifest;
pub use wam_manifest::*;

pub mod json_input;

mod asset_manager;
pub use asset_manager::{AssetManager, AssetManagerError};

slotmap::new_key_type! {
    pub struct HardAssetKey;
}

#[derive(serde::Deserialize,Debug,Copy,Clone,PartialEq,Eq)]
#[serde(rename_all = "lowercase")]
pub enum HardAssetType {
    Text,
    Image,
    Model,
}

#[derive(Debug,Clone)]
pub struct HardAsset {
    pub file_source: Rc<str>,
    pub data_type: HardAssetType,
}

pub mod reference_types {
    use super::*;

    #[derive(Debug,Clone)]
    pub struct Image {
        pub name: Rc<str>,
        pub key: HardAssetKey,
        pub size_hint: UWimpyPoint,
        pub slice: Option<WimpyPointRect>
    }

    #[derive(Debug,Clone)]
    pub struct Text {
        pub name: Rc<str>,
        pub key: HardAssetKey
    }

    #[derive(Debug)]
    pub struct Model {
        pub name: Rc<str>,
        /// The `.gltf` file that contains the mesh
        pub key: HardAssetKey,
        pub meshlet_layers: Vec<MeshletTextureLayers>,
    }

    #[derive(Debug,Copy,Clone)]
    pub struct MeshletTextureLayers {
        pub diffuse: Option<MeshletTexture>,
        pub lightmap: Option<MeshletTexture>,
    }

    #[derive(Debug)]
    pub enum MeshletField {
        Diffuse,
        Lightmap
    }

    #[derive(Debug,Copy,Clone)]
    pub struct MeshletTexture {
        pub key: HardAssetKey,
        pub size_hint: UWimpyPoint
    }
}
