use wimpy_engine::{input::InputManager, wgpu::GraphicsContext};

use crate::web_device::WebDevice;

pub struct WebApp<TWimpyApp,TConfig> {
    graphics_context: Option<GraphicsContext<WebDevice,TConfig>>,
    input_manager: InputManager,
    wimpy_app: TWimpyApp,
}
