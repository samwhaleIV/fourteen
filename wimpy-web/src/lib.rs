mod web_app;
mod web_device;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

#[wasm_bindgen(start)]
async fn main() {
    println!("Hello, world!");
}

