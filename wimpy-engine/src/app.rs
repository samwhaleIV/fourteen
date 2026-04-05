pub mod graphics;
pub mod debug_shell;
pub mod wam;
pub mod kvs;
pub mod input;
pub mod fonts;

use std::{path::Path, rc::Rc};
use graphics::{*,textures::*};
use wam::AssetManager;

use debug_shell::DebugShell;
use input::{InputManager, InputDevice};
use kvs::KeyValueStore;

#[derive(Debug,serde::Deserialize)]
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

/// Textured data expected to be provided in \[u8;4\] RGBA
/// 
/// TODO: Determine if the input format should be linear or gamma space
pub struct ImageData {
    pub size: crate::UWimpyPoint,
    pub data: Vec<u8>
}

pub trait WimpyIO {
    fn save_key_value_store(data: &[u8]) -> impl Future<Output = Result<(),FileError>>;
    fn load_key_value_store() ->            impl Future<Output = Result<Vec<u8>,FileError>>;

    fn load_binary_file(path: &Path) ->     impl Future<Output = Result<Vec<u8>,FileError>>;
    fn load_text_file(path: &Path) ->       impl Future<Output = Result<String,FileError>>;

    fn load_image_file(path: &Path) ->      impl Future<Output = Result<ImageData,FileError>>;
}

pub struct WimpyAppContext {
    pub graphics:           GraphicsContext,
    pub key_value_store:    KeyValueStore,
    pub input:              InputManager,
    pub assets:             AssetManager,
    pub debug_shell:        DebugShell,
    pub missing_text:       Rc<str>,
}

pub struct WimpyContextCreationConfig<'a> {
    pub manifest_path:          Option<&'a Path>,
    pub input_device_hint:      InputDevice,
    pub graphics_provider:      GraphicsProvider,
    pub texture_stream_policy:  StreamingPolicy
}

pub struct EngineTextures {
    pub font_classic:           WimpyTexture,
    pub font_classic_outline:   WimpyTexture,
    pub font_twelven:           WimpyTexture,
    pub font_twelven_shaded:    WimpyTexture,
    pub font_mono_elf:          WimpyTexture,
}

impl EngineTextures {
    pub fn from_placeholder(value: &WimpyTexture) -> Self {
        Self {
            font_classic:           value.clone(),
            font_classic_outline:   value.clone(),
            font_twelven:           value.clone(),
            font_twelven_shaded:    value.clone(),
            font_mono_elf:          value.clone(),
        }
    }
}

pub trait WimpyAppHandler<IO>
where
    IO: WimpyIO
{
    fn create(context: &mut WimpyAppContext) -> impl Future<Output = Self>;
    fn update(&mut self,context: &mut WimpyAppContext);
}

impl WimpyAppContext {
    pub async fn create<IO,TConfig>(config: WimpyContextCreationConfig<'_>) -> Self
    where
        IO: WimpyIO,
        TConfig: GraphicsConfig
    {
        let graphics = GraphicsContext::create::<TConfig>(
            config.graphics_provider,
            config.texture_stream_policy
        );

        let input =   input::InputManager::with_device_start_hint(config.input_device_hint);
        let storage = kvs::KeyValueStore::default();
        let debug =   debug_shell::DebugShell::default();

        let assets = AssetManager::load_or_default::<IO>(
            config.manifest_path
        ).await;

        let mut context = Self {
            graphics,
            key_value_store: storage,
            input,
            assets,
            debug_shell: debug,
            missing_text: Rc::from("<missing text asset>"),
        };

        context.graphics.texture_manager.engine_textures = EngineTextures {
            font_classic:         context.get_image( "wimpy/font/classic",        StreamingHint::Static),
            font_classic_outline: context.get_image("wimpy/font/classic-outline", StreamingHint::Static),
            font_twelven:         context.get_image("wimpy/font/twelven",         StreamingHint::Static),
            font_twelven_shaded:  context.get_image("wimpy/font/twelven-shaded",  StreamingHint::Static),
            font_mono_elf:        context.get_image("wimpy/font/mono-elf",        StreamingHint::Static),
        };

        context
    }

    // A series of assets that are 'always' expected to be a part of the runtime, such as fonts
    pub fn get_image(&mut self,name: &'static str,streaming_hint: StreamingHint) -> WimpyTexture {
        match AssetManager::get_image_asset(name,self,streaming_hint) {
            Ok(texture) => texture,
            Err(error) => {
                log::error!("Image asset load failure: {:?}",error);
                self.graphics.texture_manager.runtime_textures.missing
            },
        }
    }

    pub fn get_image_slice(&mut self,name: &'static str,streaming_hint: StreamingHint) -> WimpyTexture {
        match AssetManager::get_image_asset(name,self,streaming_hint) {
            Ok(texture) => texture,
            Err(error) => {
                log::error!("Image slice asset load failure: {:?}",error);
                self.graphics.texture_manager.runtime_textures.missing
            },
        }
    }

    pub async fn get_text<IO: WimpyIO>(&mut self,name: &'static str) -> Rc<str> {
        match AssetManager::get_text_asset::<IO>(name,self).await {
            Ok(text) => text,
            Err(error) => {
                log::error!("Text asset load failure: {:?}",error);
                self.missing_text.clone()
            },
        }
    }

    // TODO: Create fallback textured mesh inside the mesh cache
    pub async fn get_model<IO: WimpyIO>(&mut self,name: &'static str) -> Option<TexturedMesh> {
        match AssetManager::get_model_asset::<IO>(name,self).await {
            Ok(mesh) => Some(mesh),
            Err(error) => {
                log::error!("Model asset load failure: {:?}",error);
                None
            },
        }
    }
}
