use std::path::Path;

use crate::{
    input::InputManager,
    kvs::KeyValueStore,
    wam::AssetManager,
    wgpu::{
        GraphicsContext,
        TextureData
    }
};

#[derive(Debug)]
pub enum WimpyFileError {
    Access,
    Decode,
    UnsupportedFormat,
    Unknown,
}

pub trait WimpyIO {
    fn save_file(path: &Path,data: &[u8])-> impl Future<Output = Result<(),WimpyFileError>>;

    fn load_binary_file(path: &Path) -> impl Future<Output = Result<Vec<u8>,WimpyFileError>>;
    fn load_text_file(path: &Path) -> impl Future<Output = Result<String,WimpyFileError>>;

    fn load_image(path: &Path) -> impl Future<Output = Result<impl TextureData + 'static,WimpyFileError>>;

    fn save_key_value_store(kvs: &KeyValueStore) -> impl Future<Output = Result<(),WimpyFileError>>;
    fn load_key_value_store(kvs: &mut KeyValueStore) -> impl Future<Output = Result<(),WimpyFileError>>;
}

pub struct WimpyContext<'a> {
    pub graphics: &'a mut GraphicsContext,
    pub storage: &'a mut KeyValueStore,
    pub input: &'a mut InputManager,
    pub assets: &'a mut AssetManager
}

#[derive(Debug)]
pub enum WimpyAppLoadError {
    ImageError(WimpyFileError)
}

pub trait WimpyApp<IO>
where
    IO: WimpyIO
{
    fn load(&mut self,context: &WimpyContext) -> impl Future<Output = Result<(),WimpyAppLoadError>>;
    fn update(&mut self,context: &WimpyContext);
}
