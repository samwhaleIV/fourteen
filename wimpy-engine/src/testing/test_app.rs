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

const fn mb_to_b(value: usize) -> usize {
    value * 1000000
}

impl GraphicsContextConfig for PlaceholderConfig {
    // If a vertex is 32 bytes, there is 31,250 vertices per megabyte.
    const INSTANCE_BUFFER_SIZE_2D: usize = mb_to_b(10);
    const UNIFORM_BUFFER_SIZE: usize = mb_to_b(2);
    const VERTEX_BUFFER_SIZE_3D: usize = mb_to_b(10);
    const INDEX_BUFFER_SIZE_3D: usize = mb_to_b(2);
    const INSTANCE_BUFFER_SIZE_3D: usize = mb_to_b(2);
}
