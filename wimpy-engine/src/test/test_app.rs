use crate::app::*;
use crate::app::graphics::GraphicsContextConfig;

#[derive(Default)]
pub struct PlaceholderApp;

impl<IO> WimpyApp<IO> for PlaceholderApp
where
    IO: WimpyIO
{
    async fn load(&mut self,context: &WimpyContext<'_>) -> Result<(),WimpyAppLoadError> {
        return Ok(());
    }

    fn update(&mut self,context: &WimpyContext) {
        
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
    const MODEL_CACHE_VERTEX_BUFFER_SIZE: usize = mb_to_b(10);
    const MODEL_CACHE_INDEX_BUFFER_SIZE: usize = mb_to_b(2);
    const INSTANCE_BUFFER_SIZE_3D: usize = mb_to_b(2);
}
