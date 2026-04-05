const START_CACHE_ENTRY_CAPACITY: usize = 8;

use std::{path::{Path, PathBuf}, rc::Rc};
use slotmap::SparseSecondaryMap;

use crate::{UWimpyPoint, WimpyPointRect};
use crate::app::{WimpyIO, WimpyAppContext, FileError, graphics::{*, textures::*}};
use super::{*, reference_types::MeshletTexture};

#[derive(Default)]
pub struct AssetManager {
    pub manifest:   WamManifest,
    root:           PathBuf,
    text_cache:     SparseSecondaryMap<HardAssetKey,Rc<str>>,
    texture_keys:   SparseSecondaryMap<HardAssetKey,WimpyTexture>,
    model_cache:    SparseSecondaryMap<HardAssetKey,TexturedMesh>,
}

#[derive(Debug)]
pub enum AssetManagerError {
    VirtualAssetNotFound    (&'static str),
    MissingHardAsset        (&'static str),
    MismatchedType          { expected: HardAssetType, found: HardAssetType },
    FileError               (FileError),
    ModelImportError        (ModelError),
    TextureImportError      (SizeValidationError),
}

fn get_full_path(root: &PathBuf,hard_asset_path: &str) -> PathBuf {
    let mut path_buffer = PathBuf::new();
    path_buffer.push(root);
    for component in hard_asset_path.split('/') {
        path_buffer.push(component);
    }
    return path_buffer;
}

fn validate_hard_asset_type(hard_asset: &HardAsset,expected_type: HardAssetType) -> Result<(),AssetManagerError> {
    if hard_asset.data_type != expected_type {
        Err(AssetManagerError::MismatchedType {
            expected: expected_type,
            found: hard_asset.data_type 
        })
    } else {
        Ok(())
    }
}

impl AssetManager {
    pub async fn load_or_default<IO: WimpyIO>(manifest_path: Option<&Path>) -> Self {
        match manifest_path {
            Some(path) => match IO::load_text_file(path).await {
                Ok(json_text) => match WamManifest::create(&json_text) {
                    Ok(manifest) => {
                        let mut path_buffer = PathBuf::from(path);
                        path_buffer.pop();
                        Self {
                            root: path_buffer,
                            manifest: manifest,
                            text_cache: SparseSecondaryMap::with_capacity(START_CACHE_ENTRY_CAPACITY),
                            texture_keys: SparseSecondaryMap::with_capacity(START_CACHE_ENTRY_CAPACITY),
                            model_cache: SparseSecondaryMap::with_capacity(START_CACHE_ENTRY_CAPACITY),
                        }
                    },
                    Err(error) => {
                        log::error!("Could not parse manifest data '{:?}': {:?}",path,error);
                        Default::default()
                    },
                },
                Err(error) => {
                    log::error!("Could not load manifest file '{:?}': {:?}",path,error);
                    Default::default()
                },
            },
            None => Default::default(),
        }
    }

    async fn get_text_cached<IO: WimpyIO>(&mut self,key: HardAssetKey,name: &'static str) -> Result<Rc<str>,AssetManagerError> {
        if let Some(text) = self.text_cache.get(key) {
            return Ok(text.clone());
        }

        let hard_asset = match self.manifest.hard_assets.get(key) {
            Some(value) => value,
            None => return Err(AssetManagerError::MissingHardAsset(name)),
        };

        validate_hard_asset_type(hard_asset,HardAssetType::Text)?;

        let path = get_full_path(&self.root,&hard_asset.file_source);
        let text_data: Rc<str> = Rc::from(match IO::load_text_file(path.as_path()).await {
            Ok(data) => data,
            Err(error) => return Err(AssetManagerError::FileError(error)),
        });

        self.text_cache.insert(key,text_data.clone());

        Ok(text_data)
    }

    pub async fn get_text_asset<IO: WimpyIO>(name: &'static str,app: &mut WimpyAppContext) -> Result<Rc<str>,AssetManagerError> {
        let Some(virtual_asset) = app.assets.manifest.text_assets.get(name) else {
            return Err(AssetManagerError::VirtualAssetNotFound(name));
        };
        Ok(app.assets.get_text_cached::<IO>(virtual_asset.key,name).await?)
    }

    pub fn get_image_asset(name: &'static str,context: &mut WimpyAppContext,streaming_hint: StreamingHint) -> Result<WimpyTexture,AssetManagerError> {
        let Some(virtual_asset) = context.assets.manifest.image_assets.get(name) else {
            return Err(AssetManagerError::VirtualAssetNotFound(name));
        };
        let (key,size,area) = (
            virtual_asset.key,
            virtual_asset.size_hint,
            virtual_asset.slice
        );
        let mut texture_key_creator = TextureKeyCreator {
            streaming_hint,
            app: context,
        };
        Ok(texture_key_creator.create_texture(key,name,size,area))
    }

    pub async fn get_model_asset<IO: WimpyIO>(name: &'static str,app: &mut WimpyAppContext) -> Result<TexturedMesh,AssetManagerError> {

        let (hard_asset_key,meshlet_descriptors) = {
            let Some(virtual_asset) = app.assets.manifest.model_assets.get(name) else {
                return Err(AssetManagerError::VirtualAssetNotFound(name));
            };
            // I was so profoundly pissed off by the borrow checker that I threw in a clone here
            (virtual_asset.key,virtual_asset.meshlet_layers.clone())
        };

        /* We can't use 'entry()' because we mutate the slotmap cache after this to get textures */
        if let Some(mesh) = app.assets.model_cache.get(hard_asset_key) {
            return Ok(mesh.clone());
        }

        let hard_asset = match app.assets.manifest.hard_assets.get(hard_asset_key) {
            Some(value) => value,
            None => return Err(AssetManagerError::MissingHardAsset(name)),
        };
        validate_hard_asset_type(hard_asset,HardAssetType::Model)?;

        let path = get_full_path(&app.assets.root,&hard_asset.file_source);
        let gltf_data = match IO::load_binary_file(path.as_path()).await {
            Ok(data) => data,
            Err(error) => return Err(AssetManagerError::FileError(error)),
        };

        let queue = app.graphics.graphics_provider.get_queue();
        let mesh = match app.graphics.mesh_cache.insert_geometry(queue,&gltf_data) {
            Ok(value) => value,
            Err(error) => return Err(AssetManagerError::ModelImportError(error)),
        };

        // There may be more meshlet descriptions than meshlet geometry, or vice versa
        let limit = mesh.len().min(meshlet_descriptors.len());

        let mut textured_mesh: Vec<TexturedMeshlet> = Vec::with_capacity(limit);

        let mut texture_key_creator = TextureKeyCreator {
            app,
            streaming_hint: StreamingHint::Atlas,
        };

        for (i,meshlet) in mesh.into_iter().enumerate() {
            let descriptor = &meshlet_descriptors[i];

            let [diffuse,lightmap] = [descriptor.diffuse,descriptor.lightmap].map(|texture_key|{
                match texture_key {
                    Some(MeshletTexture { key, size_hint }) => texture_key_creator.create_texture(key,&name,size_hint,None),
                    None => texture_key_creator.get_missing(),
                }.key
            });

            textured_mesh.push(TexturedMeshlet {
                range: meshlet,
                diffuse,
                lightmap,
            });
        }

        let reference = app.graphics.mesh_cache.create_textured_mesh_reference(textured_mesh);
        app.assets.model_cache.insert(hard_asset_key,reference.clone());
        Ok(reference)
    }
}

struct TextureKeyCreator<'a> {
    app:            &'a mut WimpyAppContext,
    streaming_hint: StreamingHint
}

impl TextureKeyCreator<'_> {
    fn get_missing(&self) -> WimpyTexture {
        self.app.graphics.texture_manager.runtime_textures.missing.clone()
    }

    fn create_texture(
        &mut self,
        hard_asset_key: HardAssetKey,
        name: &'static str,
        size: UWimpyPoint,
        slice: Option<WimpyPointRect>
    ) -> WimpyTexture {
        if let Some(image) = self.app.assets.texture_keys.get(hard_asset_key) {
            return image.clone();
        }

        let hard_asset = match self.app.assets.manifest.hard_assets.get(hard_asset_key) {
            Some(value) => value,
            None => {
                log::error!("Hard asset key not found for '{name}'");
                return self.get_missing();
            },
        };

        if let Err(error) = validate_hard_asset_type(hard_asset,HardAssetType::Image) {
            log::error!("Texture key creation failure for key '{name}': {:?}",error);
            return self.get_missing();
        };

        let texture = self.app.graphics.texture_manager.bind_wam_asset(TextureCreationParameters {
            wam_id: hard_asset.clone(),
            policy_hint: self.streaming_hint,
            slice,
            size_hint: size,
        });

        self.app.assets.texture_keys.insert(hard_asset_key,texture.clone());

        texture
    }
}
