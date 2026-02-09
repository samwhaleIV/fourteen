mod desktop_app;
mod key_code;
mod desktop_io;

use std::{env, path::Path};

use wimpy_engine::{
    PlaceholderApp,
    PlaceholderConfig
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

    let manifest_path = Path::new(include_str!("../manifest-path.txt"));

    let mut builder = Builder::from_default_env();
    builder.target(Target::Stdout);
    builder.init();

    let wimpy_app = PlaceholderApp {};

    crate::desktop_app::run_desktop_app::<PlaceholderApp,PlaceholderConfig>(wimpy_app,Some(manifest_path));
}
