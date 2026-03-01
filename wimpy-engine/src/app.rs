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

use crate::app::graphics::{
    FrameReference, GraphicsContextConfig, GraphicsProvider, TextureFrame
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
    missing_text: &'a Rc<str>
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

        context.load_engine_textures::<IO>().await;

        Some(context)
    }

    async fn load_engine_textures<IO: WimpyIO>(&mut self) {
        use graphics::constants::assets::*;
        self.graphics.engine_textures.font_classic =         self.get_image::<IO>(FONT_CLASSIC).await;
        self.graphics.engine_textures.font_classic_outline = self.get_image::<IO>(FONT_CLASSIC_OUTLINE).await;
        self.graphics.engine_textures.font_twelven =         self.get_image::<IO>(FONT_TWELVEN).await;
        self.graphics.engine_textures.font_twelven_shaded =  self.get_image::<IO>(FONT_TWELVEN_SHADED).await;
        self.graphics.engine_textures.font_mono_elf =        self.get_image::<IO>(FONT_MONO_ELF).await;
    }

    async fn get_asset<IO,T,F>(&mut self,name: &str,fallback: F) -> T::UserAsset
    where
        IO: WimpyIO,
        T: UserAssetMapping,
        F: FnOnce(AssetLoadingContext) -> T::UserAsset
    {
        let mut context = AssetLoadingContext {
            asset_manager: &mut self.assets,
            graphics_context: &mut self.graphics,
            missing_text: &self.missing_text
        };
        let Ok(virtual_asset) = context.asset_manager.get_virtual_asset::<T::VirtualReference>(&Rc::from(name)) else {
            return fallback(context);
        };
        let Ok(user_asset) = T::get_user_asset::<IO>(virtual_asset,&mut context).await else {
            return fallback(context);
        };
        user_asset
    }

    pub async fn get_image<IO: WimpyIO>(&mut self,name: &str) -> TextureFrame {
        self.get_asset::<IO,generic_types::Image,_>(name,|context|context.graphics_context.engine_textures.missing).await
    }

    pub async fn get_image_slice<IO: WimpyIO>(&mut self,name: &str) -> ImageSliceData {
        self.get_asset::<IO,generic_types::ImageSlice,_>(name,|context|{
            let texture = context.graphics_context.engine_textures.missing;
            let size = texture.size();
            ImageSliceData {
                texture,
                area: ImageArea {
                    x: 0,
                    y: 0,
                    width: size.x,
                    height: size.y,
                }
            }
        }).await
    }

    pub async fn get_text<IO: WimpyIO>(&mut self,name: &str) -> Rc<str> {
        self.get_asset::<IO,generic_types::Text,_>(name,|context|context.missing_text.clone()).await
    }

    pub async fn get_model<IO: WimpyIO>(&mut self,name: &str) -> ModelData {
        self.get_asset::<IO,generic_types::Model,_>(name,|_|ModelData::default()).await
    }
}

pub trait WimpyApp<IO>
where
    IO: WimpyIO
{
    fn load(context: &mut WimpyContext) -> impl Future<Output = Self>;
    fn update(&mut self,context: &mut WimpyContext);
}
