use std::rc::Rc;
use serde::Deserialize;

use crate::{
    wam::*,
    wgpu::TextureFrame
};

#[derive(Debug,Default)]
pub struct HardImageAsset {
    pub state: HardAssetState<TextureFrame>
}

#[derive(Debug,Default)]
pub struct HardModelAsset {
    pub state: HardAssetState<ModelCacheReference>
}

#[derive(Debug,Default)]
pub struct HardTextAsset {
    pub state: HardAssetState<Rc<str>>
}

pub trait HardAssetResolver {
    fn type_check(asset: &mut HardAsset) -> Option<&mut Self>;
    fn get_type() -> HardAssetType;
}

impl HardAssetResolver for HardImageAsset {
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

impl HardAssetResolver for HardModelAsset {
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

impl HardAssetResolver for HardTextAsset {
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

pub trait AssetReferenceResolver<T> {
    fn type_check(asset: &AssetReference) -> Option<&T>;
}

#[derive(Debug,Clone)]
pub struct TextAssetReference {
    pub name: Rc<str>,
    pub key: HardAssetKey
}

#[derive(Debug,Clone)]
pub struct ImageAssetReference {
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
pub struct ModelAssetReference {
    pub name: Rc<str>,
    pub model: Option<HardAssetKey>,
    pub diffuse: Option<HardAssetKey>,
    pub lightmap: Option<HardAssetKey>,
}

impl AssetReferenceResolver<Self> for TextAssetReference {
    fn type_check(asset: &AssetReference) -> Option<&Self> {
        match asset {
            AssetReference::Text(virtual_text_asset) => Some(virtual_text_asset),
            _ => None
        }
    }
}

impl AssetReferenceResolver<Self> for ImageAssetReference {
    fn type_check(untyped_asset: &AssetReference) -> Option<&Self> {
        match untyped_asset {
            AssetReference::Image(image_asset) => Some(image_asset),
            _ => None
        }
    }
}

impl AssetReferenceResolver<Self> for VirtualImageSliceAsset {
    fn type_check(untyped_asset: &AssetReference) -> Option<&Self> {
        match untyped_asset {
            AssetReference::ImageSlice(image_asset) => Some(image_asset),
            _ => None
        }
    }
}

impl AssetReferenceResolver<Self> for ModelAssetReference {
    fn type_check(untyped_asset: &AssetReference) -> Option<&Self> {
        match untyped_asset {
            AssetReference::Model(model_asset) => Some(model_asset),
            _ => None
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
        return match self {
            AssetReference::Text { .. } => HardAssetType::Text,
            AssetReference::Image { .. } => HardAssetType::Image,
            AssetReference::ImageSlice { .. } => HardAssetType::Image,
            AssetReference::Model { .. } => HardAssetType::Model,
        };
    }
}
