pub mod graphics;
pub mod wam;
pub mod input;

mod debug_shell;
mod kvs;

pub use kvs::*;
use serde::Deserialize;

use std::path::Path;

use graphics::{
    TextureData,
    GraphicsContext
};

use input::{
    InputManager
};

use wam::*;

use debug_shell::DebugShell;

use crate::app::graphics::{
    GraphicsContextConfig,
    GraphicsProvider,
    TextureFrame
};

use crate::app::input::InputType;

#[derive(Debug,Deserialize)]
pub enum FileError {
    InvalidPath,
    NotFound,
    NoPermission,
    EncodeFailure,
    DecodeFailure,
    WriteFailure,
    Unknown,
    Internal,
    Other
}

pub trait WimpyIO {
    fn save_key_value_store(data: &[u8]) -> impl Future<Output = Result<(),FileError>>;
    fn load_key_value_store() -> impl Future<Output = Result<Vec<u8>,FileError>>;

    fn load_binary_file(path: &Path) -> impl Future<Output = Result<Vec<u8>,FileError>>;
    fn load_text_file(path: &Path) -> impl Future<Output = Result<String,FileError>>;

    fn load_image_file(path: &Path) -> impl Future<Output = Result<impl TextureData + 'static,FileError>>;
}

pub struct WimpyContext {
    pub graphics: GraphicsContext,
    pub storage: KeyValueStore,
    pub input: InputManager,
    pub assets: AssetManager,
    pub debug: DebugShell,
}

pub struct WimpyContextCreationConfig<'a> {
    pub manifest_path: Option<&'a Path>,
    pub input_type_hint: InputType,
    pub graphics_provider: GraphicsProvider
}

impl WimpyContext {
    pub async fn create<IO,TConfig>(config: WimpyContextCreationConfig<'_>) -> Option<Self>
    where
        IO: WimpyIO,
        TConfig: GraphicsContextConfig
    {

        let mut assets = AssetManager::load_or_default::<IO>(config.manifest_path).await;

        let graphics = GraphicsContext::create::<IO,TConfig>(
            &mut assets,
            config.graphics_provider
        ).await;

        let input = InputManager::with_input_type_hint(config.input_type_hint);
        let storage = KeyValueStore::default();
        let debug = DebugShell::default();

        return Some(Self {
            graphics,
            storage,
            input,
            assets,
            debug,
        });
    }

    async fn load_image<IO: WimpyIO>(&mut self,name: &str) -> Result<TextureFrame,AssetManagerError> {
        let reference = self.assets.get_image_reference(name)?;
        return Ok(self.assets.load_image::<IO>(&reference,&mut self.graphics).await?);
    }

    pub async fn load_image_or_default<IO: WimpyIO>(&mut self,name: &str) -> TextureFrame {
        return match self.load_image::<IO>(name).await {
            Ok(value) => value,
            Err(error) => {
                log::error!("Could not load image '{}': {:?}",name,error);
                return self.graphics.get_missing_texture();
            },
        };
    }
}

pub trait WimpyApp<IO>
where
    IO: WimpyIO
{
    fn load(context: &mut WimpyContext) -> impl Future<Output = Self>;
    fn update(&mut self,context: &mut WimpyContext);
}
