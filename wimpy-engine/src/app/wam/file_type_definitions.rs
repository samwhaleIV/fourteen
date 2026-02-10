mod image;
mod model;
mod text;

pub use image::*;
pub use model::*;
pub use text::*;

use super::prelude::*;

#[derive(Deserialize,Debug,Copy,Clone,PartialEq,Eq)]
#[serde(rename_all = "lowercase")]
pub enum HardAssetType {
    Text,
    Image,
    Model,
}


#[derive(Debug)]
pub enum HardAssetData {
    Image(HardImageAsset),
    Model(HardModelAsset),
    Text(HardTextAsset),
}

impl HardAssetData {
    pub fn get_uninit(data_type: HardAssetType) -> Self {
        match data_type {
            HardAssetType::Text => HardAssetData::Text(Default::default()),
            HardAssetType::Image => HardAssetData::Image(Default::default()),
            HardAssetType::Model => HardAssetData::Model(Default::default()),
        }
    }
}

#[derive(Debug)]
pub enum AssetReference {
    Text(TextAssetReference),
    Image(ImageAssetReference),
    ImageSlice(VirtualImageSliceAsset),
    Model(ModelAssetReference),
}

impl AssetReference {
    pub fn get_type(&self) -> HardAssetType {
        match self {
            AssetReference::Text { .. } => HardAssetType::Text,
            AssetReference::Image { .. } => HardAssetType::Image,
            AssetReference::ImageSlice { .. } => HardAssetType::Image,
            AssetReference::Model { .. } => HardAssetType::Model,
        }
    }
}
