use super::super::prelude::*;
use crate::app::graphics::TextureFrame;

#[derive(Debug,Clone)]
pub struct ImageAssetReference {
    pub name: Rc<str>,
    pub key: HardAssetKey
}

#[derive(Debug,Default)]
pub struct HardImageAsset {
    pub state: HardAssetState<TextureFrame>
}


#[derive(Debug,Clone)]
pub struct VirtualImageSliceAsset {
    pub name: Rc<str>,
    pub key: HardAssetKey,
    pub area: ImageArea
}

impl_hard_asset_resolver!(HardImageAsset,Image,HardAssetType::Image);
impl_asset_reference_resolver!(ImageAssetReference,Image);
impl_asset_reference_resolver!(VirtualImageSliceAsset,ImageSlice);
