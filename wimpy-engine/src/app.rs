pub mod graphics;
pub mod wam;
pub mod input;
pub mod kvs;

use std::path::Path;
use kvs::*;
use graphics::{
    TextureData,
    GraphicsContext
};

use input::{
    InputManager
};

use wam::*;

#[derive(Debug)]
pub enum FileError {
    Access,
    Decode,
    UnsupportedFormat,
    Unknown,
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

#[derive(Debug)]
pub enum WimpyAppLoadError {
    ImageError(FileError)
}

pub trait WimpyApp<IO>
where
    IO: WimpyIO
{
    fn load(&mut self,context: &WimpyContext) -> impl Future<Output = Result<(),WimpyAppLoadError>>;
    fn update(&mut self,context: &WimpyContext);
}
