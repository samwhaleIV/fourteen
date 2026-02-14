pub mod graphics;
pub mod wam;
pub mod input;
mod kvs;

pub use kvs::*;

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

#[derive(Debug)]
pub enum FileError {
    Access,
    Decode,
    UnsupportedFormat,
    Unknown,
    InvalidPath
}

pub trait WimpyIO {
    fn save_file(path: &Path,data: &[u8])-> impl Future<Output = Result<(),FileError>>;

    fn load_binary_file(path: &Path) -> impl Future<Output = Result<Vec<u8>,FileError>>;
    fn load_text_file(path: &Path) -> impl Future<Output = Result<String,FileError>>;

    fn load_image_file(path: &Path) -> impl Future<Output = Result<impl TextureData + 'static,FileError>>;

    fn save_key_value_store(kvs: &KeyValueStore) -> impl Future<Output = Result<(),FileError>>;
    fn load_key_value_store(kvs: &mut KeyValueStore) -> impl Future<Output = Result<(),FileError>>;
}

pub struct WimpyContext<'a> {
    pub graphics: &'a mut GraphicsContext,
    pub storage: &'a mut KeyValueStore,
    pub input: &'a mut InputManager,
    pub assets: &'a mut AssetManager
}

impl WimpyContext<'_> {
    pub async fn load_image_or_default<IO: WimpyIO>(&mut self,name: &str) -> TextureFrame {
        let reference = match self.assets.get_image_reference(name) {
            Ok(value) => value,
            Err(error) => {
                log::error!("Could not load image: {:?}",error);
                return self.graphics.get_missing_texture();
            },
        };
        return match self.assets.load_image::<IO>(&reference,self.graphics).await {
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
