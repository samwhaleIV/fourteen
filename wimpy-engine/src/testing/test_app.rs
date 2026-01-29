use crate::{
    WimpyApp,
    WimpyAppLoadError,
    WimpyContext,
    WimpyIO,
    wgpu::GraphicsContextConfig
};

pub struct PlaceholderApp {}

impl<IO,Config> WimpyApp<IO,Config> for PlaceholderApp
where
    IO: WimpyIO
{
    async fn load(&mut self,context: &WimpyContext<'_,Config>) -> Result<(),WimpyAppLoadError> {
        return Ok(());
    }

    fn update(&mut self,context: &WimpyContext<Config>) {
        
    }
}

pub struct PlaceholderConfig;

impl GraphicsContextConfig for PlaceholderConfig {
    const INSTANCE_CAPACITY: usize = 1000;
    const UNIFORM_CAPACITY: usize = 16;
}
