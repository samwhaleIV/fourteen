mod web_app;
mod key_code;
mod wimpy_web_io;
mod gamepad_manager;

pub use web_app::*;
pub use key_code::*;
pub use wimpy_web_io::*;
pub use gamepad_manager::*;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

use wimpy_engine::test::*;

const MANIFEST_PATH: &'static str = "assets/wam.json";

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
fn main() {
    console_log::init_with_level(log::Level::Trace).unwrap_throw();
    console_error_panic_hook::set_once();
    spawn_local(async {
        use std::path::Path;

        let manifest_path = Path::new(MANIFEST_PATH);

        let _ = match web_app::WebApp::<SrgbTest>::run::<TestConfig>(Some(manifest_path),web_app::ResizeConfig::FitWindow).await {
            Ok(app) => app,
            Err(error) => {
                log::error!("Could not create wimpy web app: {:?}",error);
                return;
            },
        };
    });
}
