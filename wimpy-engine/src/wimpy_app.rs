use crate::{
    input::InputManager,
    storage::KeyValueStore,
    wam::WamManifest,
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
    fn save_key_value_store(kvs: &KeyValueStore);
    fn load_key_value_store(kvs: &mut KeyValueStore);
    fn get_image(path: &'static str) -> impl Future<Output = Result<impl TextureData,WimpyFileError>>;
    fn get_text(path: &'static str) -> impl Future<Output = Result<String,WimpyFileError>>;
}

pub struct WimpyContext<'a> {
    pub graphics: &'a mut GraphicsContext,
    pub storage: &'a mut KeyValueStore,
    pub input: &'a mut InputManager,
    pub manifest: &'a mut WamManifest
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
