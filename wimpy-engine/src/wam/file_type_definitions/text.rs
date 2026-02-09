use std::rc::Rc;

use crate::{
    impl_asset_reference_resolver,
    impl_hard_asset_resolver,
    wam::*
};

#[derive(Debug,Clone)]
pub struct TextAssetReference {
    pub name: Rc<str>,
    pub key: HardAssetKey
}
impl_asset_reference_resolver!(TextAssetReference,Text);

#[derive(Debug,Default)]
pub struct HardTextAsset {
    pub state: HardAssetState<Rc<str>>
}
impl_hard_asset_resolver!(HardTextAsset,Text,HardAssetType::Text);

