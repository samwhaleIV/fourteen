mod desktop_app;
mod key_code;

use std::env;

use wimpy_engine::{
    PlaceholderConfig,
    PlaceholderApp
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

    let mut builder = Builder::from_default_env();
    builder.target(Target::Stdout);
    builder.init();

    let wimpy_app = PlaceholderApp {};

    crate::desktop_app::run_desktop_app::<PlaceholderApp,PlaceholderConfig>(wimpy_app);
}
