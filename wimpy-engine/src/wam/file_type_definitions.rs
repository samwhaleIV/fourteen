use serde::Deserialize;

use crate::{wam::*, wgpu::FrameCacheReference};

#[derive(Debug,Default)]
pub struct HardImageAsset(pub AssetState<FrameCacheReference>);

#[derive(Debug,Default)]
pub struct HardModelAsset(pub AssetState<ModelCacheReference>);

#[derive(Debug,Default)]
pub struct HardTextAsset(pub AssetState<String>);

impl DataResolver<Self> for HardImageAsset {
    fn resolve_asset(asset: &HardAsset) -> Option<&Self> {
        return match &asset.data {
            HardAssetData::Image(data) => Some(data),
            _ => None
        }
    }
    
    fn get_type() -> HardAssetType {
        return HardAssetType::Image;
    }
}

impl DataResolver<Self> for HardModelAsset {
    fn resolve_asset(asset: &HardAsset) -> Option<&Self> {
        return match &asset.data {
            HardAssetData::Model(data) => Some(data),
            _ => None
        }
    }
    fn get_type() -> HardAssetType {
        return HardAssetType::Model;
    }
}

impl DataResolver<Self> for HardTextAsset {
    fn resolve_asset(asset: &HardAsset) -> Option<&Self> {
        return match &asset.data {
            HardAssetData::Text(data) => Some(data),
            _ => None
        }
    }
    fn get_type() -> HardAssetType {
        return HardAssetType::Text;
    }
}

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
        return match data_type {
            HardAssetType::Text => HardAssetData::Text(Default::default()),
            HardAssetType::Image => HardAssetData::Image(Default::default()),
            HardAssetType::Model => HardAssetData::Model(Default::default()),
        }
    }
}

#[derive(Debug)]
pub struct VirtualModelData {
    pub model: Option<HardAssetKey>,
    pub diffuse: Option<HardAssetKey>,
    pub lightmap: Option<HardAssetKey>,
}

#[derive(Debug)]
pub enum VirtualAsset {
    Text(HardAssetKey),
    Image(HardAssetKey),
    Model(VirtualModelData),
    ImageSlice {
        key: HardAssetKey,
        area: ImageArea
    },
}

impl VirtualAsset {
    pub fn get_type(&self) -> HardAssetType {
        return match self {
            VirtualAsset::Text { .. } => HardAssetType::Text,
            VirtualAsset::Image { .. } => HardAssetType::Image,
            VirtualAsset::ImageSlice { .. } => HardAssetType::Image,
            VirtualAsset::Model { .. } => HardAssetType::Model,
        };
    }
}
