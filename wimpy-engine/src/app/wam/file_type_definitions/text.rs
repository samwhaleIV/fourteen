use super::*;

#[derive(Debug,Clone)]
pub struct TextAssetReference {
    pub name: Rc<str>,
    pub key: HardAssetKey
}

#[derive(Debug,Default)]
pub struct HardTextAsset {
    pub state: HardAssetState<Rc<str>>
}

impl_asset_reference_resolver!(TextAssetReference,Text);
impl_hard_asset_resolver!(HardTextAsset,Text,HardAssetType::Text);

