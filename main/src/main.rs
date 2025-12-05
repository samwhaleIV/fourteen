mod app;
mod test_state;
mod graphics;
pub mod point;

use std::env;
use winit::{event_loop::{ControlFlow,EventLoop}};

use crate::test_state::TestState;

use env_logger::{Builder, Target};

fn create_event_loop() -> anyhow::Result<()> {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = app::create_app(test_state::generate_test_state);

    event_loop.run_app(&mut app)?;

    return Ok(());
}

pub fn main() {
    let log_variable = "RUST_LOG";

    match env::var(log_variable) {
        Ok(value) => println!("{}: {:?}",log_variable,value),
        Err(error) => println!("Error {}: {}",log_variable,error),
    }

    let mut builder = Builder::from_default_env();
    builder.target(Target::Stdout);
    builder.init();

    log::info!("Logger initialized, you bet.");

    let _ = create_event_loop();
}
