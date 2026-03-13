use super::prelude::*;

pub struct TextAssetKey {

}

#[derive(Debug)]
pub struct MeshletDescriptor {
    pub diffuse: Option<HardAssetKey>,
    pub lightmap: Option<HardAssetKey>,
}

#[derive(Debug)]
pub struct ModelAssetReference {
    pub name: Rc<str>,
    /// The `.gltf` file that contains the mesh
    pub key: HardAssetKey,
    pub meshlet_descriptors: Vec<MeshletDescriptor>,
}

#[derive(Debug,Clone)]
pub struct ImageAssetReference {
    pub name: Rc<str>,
    pub key: HardAssetKey
}

#[derive(Debug,Clone)]
pub struct ImageSliceAssetReference {
    pub name: Rc<str>,
    pub key: HardAssetKey,
    pub area: ImageArea
}

#[derive(Debug,Clone)]
pub struct TextAssetReference {
    pub name: Rc<str>,
    pub key: HardAssetKey
}


#[derive(Deserialize,Debug,Copy,Clone,PartialEq,Eq)]
#[serde(rename_all = "lowercase")]
pub enum HardAssetType {
    Text,
    Image,
    Model,
}

#[derive(Debug)]
pub enum AssetReference {
    Text(TextAssetReference),
    Image(ImageAssetReference),
    ImageSlice(ImageSliceAssetReference),
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

impl_asset_reference_resolver!(ModelAssetReference,Model);
impl_asset_reference_resolver!(ImageAssetReference,Image);
impl_asset_reference_resolver!(ImageSliceAssetReference,ImageSlice);
impl_asset_reference_resolver!(TextAssetReference,Text);
