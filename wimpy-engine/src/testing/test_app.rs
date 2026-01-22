use crate::{
    WimpyApp,
    WimpyContext, 
    WimpyIO,
    wgpu::GraphicsContextConfig
};

pub struct PlaceholderApp {}

impl<IO,Config> WimpyApp<IO,Config> for PlaceholderApp
where
    IO: WimpyIO
{
    fn render(&mut self,context: &WimpyContext<Config>) {
        
    }
}

pub struct PlaceholderConfig;

impl GraphicsContextConfig for PlaceholderConfig {
    const INSTANCE_CAPACITY: usize = 1000;
    const UNIFORM_CAPACITY: usize = 16;
    const CACHE_INSTANCES: usize = 8;
    const CACHE_SIZES: &[(u32,u32)] = &[(64,64)];
}
