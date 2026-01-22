use crate::{
    input::InputManager,
    storage::KeyValueStore, 
    wgpu::GraphicsContext
};

pub enum WimpyIOError {

}

pub trait WimpyIO {
    fn save_key_value_store(kvs: &KeyValueStore);
    fn load_key_value_store(kvs: &mut KeyValueStore);
    fn get_file_bytes(file: &'static str) -> Result<Vec<u8>,WimpyIOError>;
}

pub struct WimpyContext<'a,TConfig> {
    pub graphics: &'a mut GraphicsContext<TConfig>,
    pub storage: &'a mut KeyValueStore,
    pub input: &'a mut InputManager,
}

pub trait WimpyApp<IO,Config> where IO: WimpyIO {
    fn render(&mut self,context: &WimpyContext<Config>);
}
