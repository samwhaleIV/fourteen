use std::rc::Rc;

use crate::{
    impl_asset_reference_resolver,
    impl_hard_asset_resolver,
    wam::*
};

#[derive(Debug,Default)]
pub struct HardModelAsset {
    pub state: HardAssetState<ModelCacheReference>
}

#[derive(Debug,Clone)]
pub struct ModelAssetReference {
    pub name: Rc<str>,
    pub model: Option<HardAssetKey>,
    pub diffuse: Option<HardAssetKey>,
    pub lightmap: Option<HardAssetKey>,
}

impl_asset_reference_resolver!(ModelAssetReference,Model);
impl_hard_asset_resolver!(HardModelAsset,Model,HardAssetType::Model);
