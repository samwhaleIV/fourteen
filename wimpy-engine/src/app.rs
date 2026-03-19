pub mod graphics;
pub mod debug_shell;    
pub mod wam;
pub mod kvs;
pub mod input;
pub mod fonts;

use std::{path::Path, rc::Rc};
use graphics::{GraphicsConfig, GraphicsProvider, GraphicsContext, mesh_cache::TexturedMesh};
use graphics::textures::{WimpyTexture, StreamingHint, StreamingPolicy};

use wam::{AssetManager, AssetManagerError};

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

const MISSING_TEXT_ASSET: &'static str = "<missing text asset>";


/// Textured data expected to be provided in [u8;4] RGBA
/// 
/// TODO: Determine if the input format should be linear or gamma space
pub struct ImageData {
    pub size: crate::UWimpyPoint,
    pub data: Vec<u8>
}

pub trait WimpyIO {
    fn save_key_value_store(data: &[u8]) -> impl Future<Output = Result<(),FileError>>;
    fn load_key_value_store() -> impl Future<Output = Result<Vec<u8>,FileError>>;

    fn load_binary_file(path: &Path) -> impl Future<Output = Result<Vec<u8>,FileError>>;
    fn load_text_file(path: &Path) -> impl Future<Output = Result<String,FileError>>;

    fn load_image_file(path: &Path) -> impl Future<Output = Result<ImageData,FileError>>;
}

pub struct WimpyContext {
    pub graphics: GraphicsContext,
    pub storage: KeyValueStore,
    pub input: InputManager,
    pub assets: AssetManager,
    pub debug: DebugShell,
    missing_text: Rc<str>,
}

pub struct WimpyContextCreationConfig<'a> {
    pub manifest_path: Option<&'a Path>,
    pub input_device_hint: InputDevice,
    pub graphics_provider: GraphicsProvider,
    pub texture_stream_policy: StreamingPolicy
}

pub struct AssetLoadingContext<'a> {
    assets: &'a mut wam::AssetManager,
    graphics: &'a mut GraphicsContext,
    texture_streaming_hint: StreamingHint
}

impl<'a> AssetLoadingContext<'a> {
    fn create(app: &'a mut WimpyContext,texture_streaming_hint: StreamingHint) -> Self {
        Self {
            assets: &mut app.assets,
            graphics: &mut app.graphics,
            texture_streaming_hint,
        }
    }
}

impl<'a> From<&'a mut WimpyContext> for AssetLoadingContext<'a> {
    fn from(value: &'a mut WimpyContext) -> Self {
        AssetLoadingContext {
            assets: &mut value.assets,
            graphics: &mut value.graphics,
            texture_streaming_hint: StreamingHint::None,
        }
    }
}

impl WimpyContext {
    pub async fn create<IO,TConfig>(config: WimpyContextCreationConfig<'_>) -> Option<Self>
    where
        IO: WimpyIO,
        TConfig: GraphicsConfig
    {
        let assets = AssetManager::load_or_default::<IO>(
            config.manifest_path
        ).await;

        let graphics = GraphicsContext::create::<TConfig>(
            config.graphics_provider,
            config.texture_stream_policy
        ).await;

        let input = input::InputManager::with_input_type_hint(config.input_device_hint);
        let storage = kvs::KeyValueStore::default();
        let debug = debug_shell::DebugShell::default();

        let mut context = Self {
            graphics,
            storage,
            input,
            assets,
            debug,
            missing_text: Rc::from(MISSING_TEXT_ASSET),
        };

        context.bind_engine_assets();

        Some(context)
    }

    // A series of assets that are 'always' expected to be a part of the runtime, such as fonts
    fn bind_engine_assets(&mut self) {
        use graphics::constants::assets::*;
        self.graphics.engine_textures.font_classic =         self.get_image(FONT_CLASSIC,           StreamingHint::Static);
        self.graphics.engine_textures.font_classic_outline = self.get_image(FONT_CLASSIC_OUTLINE,   StreamingHint::Static);
        self.graphics.engine_textures.font_twelven =         self.get_image(FONT_TWELVEN,           StreamingHint::Static);
        self.graphics.engine_textures.font_twelven_shaded =  self.get_image(FONT_TWELVEN_SHADED,    StreamingHint::Static);
        self.graphics.engine_textures.font_mono_elf =        self.get_image(FONT_MONO_ELF,          StreamingHint::Static);
    }

    pub fn get_image(&mut self,name: &'static str,streaming_hint: StreamingHint) -> WimpyTexture {
        match AssetManager::get_image_asset(name,&mut AssetLoadingContext::create(self,streaming_hint)) {
            Ok(texture) => texture,
            Err(error) => {
                log::error!("Image asset load failure: {:?}",error);
                self.graphics.engine_textures.missing
            },
        }
    }

    pub fn get_image_slice(&mut self,name: &'static str,streaming_hint: StreamingHint) -> WimpyTexture {
        match AssetManager::get_image_asset(name,&mut AssetLoadingContext::create(self,streaming_hint)) {
            Ok(texture) => texture,
            Err(error) => {
                log::error!("Image slice asset load failure: {:?}",error);
                self.graphics.engine_textures.missing
            },
        }
    }

    pub async fn get_text<IO: WimpyIO>(&mut self,name: &'static str) -> Rc<str> {
        match AssetManager::get_text_asset::<IO>(name,&mut self.into()).await {
            Ok(text) => text,
            Err(error) => {
                log::error!("Text asset load failure: {:?}",error);
                self.missing_text.clone()
            },
        }
    }

    // TODO: Create fallback textured mesh inside the mesh cache
    pub async fn get_model<IO: WimpyIO>(&mut self,name: &'static str) -> Option<TexturedMesh> {
        match AssetManager::get_model_asset::<IO>(name,&mut self.into()).await {
            Ok(mesh) => Some(mesh),
            Err(error) => {
                log::error!("Model asset load failure: {:?}",error);
                None
            },
        }
    }
}

pub trait WimpyApp<IO>
where
    IO: WimpyIO
{
    fn load(context: &mut WimpyContext) -> impl Future<Output = Self>;
    fn update(&mut self,context: &mut WimpyContext);
}
