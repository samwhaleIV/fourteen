pub mod graphics;
pub mod wam;
pub mod input;

mod debug_shell;
pub use debug_shell::*;

mod kvs;

pub use kvs::*;
use serde::Deserialize;

use std::path::Path;
use std::rc::Rc;

use graphics::{
    TextureData,
    GraphicsContext
};

use input::InputManager;

use wam::*;

use crate::app::graphics::{FrameReference, GraphicsContextConfig, GraphicsProvider, TextureFrame, TexturedMeshReference};

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

const MISSING_TEXT_ASSET: &'static str = "<missing text asset>";

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
    missing_text: Rc<str>,
}

pub struct WimpyContextCreationConfig<'a> {
    pub manifest_path: Option<&'a Path>,
    pub input_type_hint: InputType,
    pub graphics_provider: GraphicsProvider
}

pub struct AssetLoadingContext<'a> {
    asset_manager: &'a mut AssetManager,
    graphics_context: &'a mut GraphicsContext,
}

impl WimpyContext {
    pub async fn create<IO,TConfig>(config: WimpyContextCreationConfig<'_>) -> Option<Self>
    where
        IO: WimpyIO,
        TConfig: GraphicsContextConfig
    {
        let assets = AssetManager::load_or_default::<IO>(config.manifest_path).await;
        let graphics = GraphicsContext::create::<TConfig>(config.graphics_provider).await;

        let input = InputManager::with_input_type_hint(config.input_type_hint);
        let storage = KeyValueStore::default();
        let debug = DebugShell::default();

        let mut context = Self {
            graphics,
            storage,
            input,
            assets,
            debug,
            missing_text: Rc::from(MISSING_TEXT_ASSET),
        };

        context.load_engine_assets::<IO>().await;

        Some(context)
    }

    // A series of assets that are 'always' expected to be a part of the runtime, such as fonts
    async fn load_engine_assets<IO: WimpyIO>(&mut self) {
        use graphics::constants::assets::*;
        self.graphics.engine_textures.font_classic =         self.get_image::<IO>(FONT_CLASSIC).await;
        self.graphics.engine_textures.font_classic_outline = self.get_image::<IO>(FONT_CLASSIC_OUTLINE).await;
        self.graphics.engine_textures.font_twelven =         self.get_image::<IO>(FONT_TWELVEN).await;
        self.graphics.engine_textures.font_twelven_shaded =  self.get_image::<IO>(FONT_TWELVEN_SHADED).await;
        self.graphics.engine_textures.font_mono_elf =        self.get_image::<IO>(FONT_MONO_ELF).await;
    }

    pub async fn get_image<IO: WimpyIO>(&mut self,name: &'static str) -> TextureFrame {
        match ImageAssetReference::get_cached::<IO>(name,&mut self.into()).await {
            Ok(frame_view) => frame_view.texture,
            Err(error) => {
                log::error!("Image asset load failure: {:?}",error);
                self.graphics.engine_textures.missing
            },
        }
    }

    pub async fn get_image_slice<IO: WimpyIO>(&mut self,name: &'static str) -> TextureFrameView {
        match ImageAssetReference::get_cached::<IO>(name,&mut self.into()).await {
            Ok(frame_view) => frame_view,
            Err(error) => {
                log::error!("Image slice asset load failure: {:?}",error);
                let texture = self.graphics.engine_textures.missing;
                let image_area = texture.get_input_size();
                TextureFrameView {
                    texture,
                    view: Some(ImageArea {
                        x: 0,
                        y: 0,
                        width: image_area.x,
                        height: image_area.y,
                    }),
                }
            },
        }
    }

    pub async fn get_text<IO: WimpyIO>(&mut self,name: &'static str) -> Rc<str> {
        match TextAssetReference::get_cached::<IO>(name,&mut self.into()).await {
            Ok(text) => text,
            Err(error) => {
                log::error!("Text asset load failure: {:?}",error);
                self.missing_text.clone()
            },
        }
    }

    // TODO: Create fallback textured mesh inside the mesh cache
    pub async fn get_model<IO: WimpyIO>(&mut self,name: &'static str) -> Option<TexturedMeshReference> {
        match ModelAssetReference::get_cached::<IO>(name,&mut self.into()).await {
            Ok(mesh) => Some(mesh),
            Err(error) => {
                log::error!("Model asset load failure: {:?}",error);
                None
            },
        }
    }
}

impl<'a> From<&'a mut WimpyContext> for AssetLoadingContext<'a> {
    fn from(value: &'a mut WimpyContext) -> Self {
        AssetLoadingContext {
            asset_manager: &mut value.assets,
            graphics_context: &mut value.graphics,
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
