mod app_state;
mod app;
mod graphics;
mod test_state;

use std::env;
use wimpy::pipeline_management::PipelineCreationOptions;
use winit::{event_loop::{ControlFlow,EventLoop,DeviceEvents}};
use env_logger::{Builder, Target};

use crate::app::AppCreationOptions;

fn create_event_loop() -> anyhow::Result<()> {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = app::create_app(AppCreationOptions {
        state_generator: test_state::generate_test_state,
        pipeline_options: Some(PipelineCreationOptions {
            quad_instance_capacity: 1000000,
            uniform_capacity: 64,
            cache_options: None,
        }),
        log_trace_config: None
    });

    event_loop.listen_device_events(DeviceEvents::WhenFocused);

    log::info!("Starting event loop! Here we go. No going back now.");

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

    let _ = create_event_loop();
}
