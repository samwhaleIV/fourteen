use std::rc::Rc;

use serde::Deserialize;

use crate::{wam::*, wgpu::FrameCacheReference};

#[derive(Debug,Default)]
pub struct HardImageAsset {
    pub state: AssetState<FrameCacheReference>
}

#[derive(Debug,Default)]
pub struct HardModelAsset {
    pub state: AssetState<ModelCacheReference>
}

#[derive(Debug,Default)]
pub struct HardTextAsset {
    pub state: AssetState<String>
}

pub trait DataResolver<T> {
    fn type_check(asset: &mut HardAsset) -> Option<&mut T>;
    fn get_type() -> HardAssetType;
}

impl DataResolver<Self> for HardImageAsset {
    fn type_check(asset: &mut HardAsset) -> Option<&mut Self> {
        return match &mut asset.data {
            HardAssetData::Image(data) => Some(data),
            _ => None
        }
    }
    
    fn get_type() -> HardAssetType {
        return HardAssetType::Image;
    }
}

impl DataResolver<Self> for HardModelAsset {
    fn type_check(asset: &mut HardAsset) -> Option<&mut Self> {
        return match &mut asset.data {
            HardAssetData::Model(data) => Some(data),
            _ => None
        }
    }
    fn get_type() -> HardAssetType {
        return HardAssetType::Model;
    }
}

impl DataResolver<Self> for HardTextAsset {
    fn type_check(asset: &mut HardAsset) -> Option<&mut Self> {
        return match &mut asset.data {
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

pub trait VirtualAssetResolver<T> {
    fn type_check(asset: &VirtualAsset) -> Option<&T>;
}

#[derive(Debug,Clone)]
pub struct VirtualTextAsset {
    pub name: Rc<str>,
    pub key: HardAssetKey
}

#[derive(Debug,Clone)]
pub struct VirtualImageAsset {
    pub name: Rc<str>,
    pub key: HardAssetKey
}

#[derive(Debug,Clone)]
pub struct VirtualImageSliceAsset {
    pub name: Rc<str>,
    pub key: HardAssetKey,
    pub area: ImageArea
}

#[derive(Debug,Clone)]
pub struct VirtualModelAsset {
    pub name: Rc<str>,
    pub model: Option<HardAssetKey>,
    pub diffuse: Option<HardAssetKey>,
    pub lightmap: Option<HardAssetKey>,
}

impl VirtualAssetResolver<Self> for VirtualTextAsset {
    fn type_check(asset: &VirtualAsset) -> Option<&Self> {
        match asset {
            VirtualAsset::Text(virtual_text_asset) => Some(virtual_text_asset),
            _ => None
        }
    }
}

impl VirtualAssetResolver<Self> for VirtualImageAsset {
    fn type_check(untyped_asset: &VirtualAsset) -> Option<&Self> {
        match untyped_asset {
            VirtualAsset::Image(image_asset) => Some(image_asset),
            _ => None
        }
    }
}

impl VirtualAssetResolver<Self> for VirtualImageSliceAsset {
    fn type_check(untyped_asset: &VirtualAsset) -> Option<&Self> {
        match untyped_asset {
            VirtualAsset::ImageSlice(image_asset) => Some(image_asset),
            _ => None
        }
    }
}

impl VirtualAssetResolver<Self> for VirtualModelAsset {
    fn type_check(untyped_asset: &VirtualAsset) -> Option<&Self> {
        match untyped_asset {
            VirtualAsset::Model(model_asset) => Some(model_asset),
            _ => None
        }
    }
}

#[derive(Debug)]
pub enum VirtualAsset {
    Text(VirtualTextAsset),
    Image(VirtualImageAsset),
    ImageSlice(VirtualImageSliceAsset),
    Model(VirtualModelAsset),
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
