mod web_app;
mod key_code;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use wimpy_engine::wgpu::GraphicsContextConfig;

struct PlaceholderApp;

impl wimpy_engine::WimpyAppHandler for PlaceholderApp {

}

struct EngineConfig;

impl GraphicsContextConfig for EngineConfig {
    const INSTANCE_CAPACITY: usize = 1000;
    const UNIFORM_CAPACITY: usize = 16;
    const CACHE_INSTANCES: usize = 8;
    const CACHE_SIZES: &[(u32,u32)] = &[(64,64)];
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
fn main() {
    console_log::init_with_level(log::Level::Trace).unwrap_throw();
    console_error_panic_hook::set_once();
    spawn_local(async {
        let wimpy_app = PlaceholderApp {};
        let _ = match web_app::WebApp::<PlaceholderApp,EngineConfig>::run(wimpy_app,web_app::ResizeConfig::FitWindow).await {
            Ok(app) => app,
            Err(error) => {
                log::error!("Could not create wimpy web app: {:?}",error);
                return;
            },
        };
    });
}
