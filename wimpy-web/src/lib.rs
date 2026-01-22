mod web_app;
mod key_code;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

use wimpy_engine::{
    PlaceholderApp,
    PlaceholderConfig,
};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
fn main() {
    console_log::init_with_level(log::Level::Trace).unwrap_throw();
    console_error_panic_hook::set_once();
    spawn_local(async {

        let wimpy_app = PlaceholderApp {};
        let _ = match web_app::WebApp::<PlaceholderApp,PlaceholderConfig>::run(wimpy_app,web_app::ResizeConfig::FitWindow).await {
            Ok(app) => app,
            Err(error) => {
                log::error!("Could not create wimpy web app: {:?}",error);
                return;
            },
        };
    });
}
