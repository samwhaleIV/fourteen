use serde::Deserialize;

#[derive(Deserialize,Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Namespace {
    pub hard_assets: Vec<HardAsset>,
    pub image_size_hints: Vec<SizeHint>,
    pub virtual_assets: Vec<VirtualAsset>,
    pub virtual_image_slice_assets: Vec<VirtualImageAsset>,
    pub virtual_model_assets: Vec<VirtualModelAsset>
}

#[derive(Deserialize,Debug)]
#[serde(rename_all = "lowercase")]
pub struct HardAsset {
    pub id: u32,
    pub source: String,
    pub r#type: super::HardAssetType
}

#[derive(Deserialize,Debug)]
#[serde(rename_all = "lowercase")]
pub struct VirtualAsset {
    pub id: u32,
    pub name: String,
}

#[derive(Deserialize,Debug)]
#[serde(rename_all = "lowercase")]
pub struct VirtualImageAsset {
    pub id: u32,
    pub name: String,
    pub slice: crate::WimpyPointRect
}

#[derive(Deserialize,Debug)]
#[serde(rename_all = "lowercase")]
pub struct MeshletDescriptor {
    pub diffuse: Option<u32>,
    pub lightmap: Option<u32>,
}

#[derive(Deserialize,Debug)]
#[serde(rename_all = "lowercase")]
pub struct VirtualModelAsset {
    pub id: u32,
    pub name: String,
    #[serde(default)]
    pub meshlets: Vec<MeshletDescriptor>
}

#[derive(Deserialize,Debug)]
#[serde(rename_all = "lowercase")]
pub struct SizeHint {
    pub id: u32,
    pub x: u32,
    pub y: u32,
}
