use super::prelude::*;

#[derive(Deserialize,Debug,Copy,Clone,PartialEq,Eq)]
#[serde(rename_all = "lowercase")]
pub enum HardAssetType {
    Text,
    Image,
    Model,
}

#[derive(Debug,Copy,Clone)]
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
    pub key: HardAssetKey,
    pub area: Option<ImageArea>
}

#[derive(Debug,Clone)]
pub struct TextAssetReference {
    pub name: Rc<str>,
    pub key: HardAssetKey
}
