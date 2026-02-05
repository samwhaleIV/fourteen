mod desktop_app;
mod key_code;

use std::{env, fs};

use wimpy_engine::{
    PlaceholderApp, PlaceholderConfig, wam
};

use env_logger::{
    Builder,
    Target
};

pub fn main() {
    let log_variable = "RUST_LOG";

    match env::var(log_variable) {
        Ok(value) => println!("{}: {:?}",log_variable,value),
        Err(error) => println!("Error {}: {}",log_variable,error),
    }

    let manifest_path = "C:\\Users\\pinks\\OneDrive\\Documents\\Rust Projects\\fourteen\\assets\\debug-output\\manifest.json";
    match wam::load_manifest_from_path(manifest_path) {
        Ok(manifest) => {
            println!("{:?}",manifest);
        },
        Err(error) => {
            println!("bad manifest: {:?}",error);
            return
        },
    };

    return;

    let mut builder = Builder::from_default_env();
    builder.target(Target::Stdout);
    builder.init();

    let wimpy_app = PlaceholderApp {};

    crate::desktop_app::run_desktop_app::<PlaceholderApp,PlaceholderConfig>(wimpy_app);
}
