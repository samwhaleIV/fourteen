pub mod graphics;
pub mod wam;
pub mod input;
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

use crate::app::graphics::TextureFrame;

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

pub struct WimpyContext<'a> {
    pub graphics: &'a mut GraphicsContext,
    pub storage: &'a mut KeyValueStore,
    pub input: &'a mut InputManager,
    pub assets: &'a mut AssetManager
}

impl WimpyContext<'_> {

    async fn load_image<IO: WimpyIO>(&mut self,name: &str) -> Result<TextureFrame,AssetManagerError> {
        let reference = self.assets.get_image_reference(name)?;
        return Ok(self.assets.load_image::<IO>(&reference,self.graphics).await?);
    }

    pub async fn load_image_or_default<IO: WimpyIO>(&mut self,name: &str) -> TextureFrame {
        return match self.load_image::<IO>(name).await {
            Ok(value) => value,
            Err(error) => {
                log::error!("Could not load image: {:?}",error);
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
