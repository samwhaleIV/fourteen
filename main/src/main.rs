mod test_state;

use env_logger::{
    Builder,
    Target
};

use std::env;
use wimpy::graphics::GraphicsContextConfiguration;

use winit::event_loop::{
    ControlFlow,
    DeviceEvents,
    EventLoop
};

use wimpy::app::{
    App,
    AppConfiguration
};

use crate::test_state::{
    SharedState,
    generate_test_state
};

fn create_event_loop() -> anyhow::Result<()> {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::create(AppConfiguration::<SharedState> {
        state_generator: generate_test_state,
        shared_state_generator: SharedState::generator,
        context_options: Some(GraphicsContextConfiguration {
            quad_instance_capacity: 10000,
            uniform_capacity: 32,
            cache_options: None,
        }),
        log_trace_config: None,
    });

    event_loop.listen_device_events(DeviceEvents::WhenFocused);

    log::info!("Starting event loop! Here we go. No going back now.");

    event_loop.run_app(&mut app)?;
    return Ok(());
}

pub fn main() {
    let log_variable = "RUST_LOG";

    match env::var(log_variable) {
        Ok(value) => println!("{}: {:?}", log_variable, value),
        Err(error) => println!("Error {}: {}", log_variable, error),
    }

    let mut builder = Builder::from_default_env();
    builder.target(Target::Stdout);
    builder.init();

    let _ = create_event_loop();
}
