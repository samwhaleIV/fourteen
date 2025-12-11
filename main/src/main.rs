use std::env;
use winit::{event_loop::{ControlFlow,EventLoop,DeviceEvents}};
use env_logger::{Builder, Target};

fn create_event_loop() -> anyhow::Result<()> {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = shared::app::create_app(
        shared::test_state::generate_test_state,
        shared::app::LogTraceConfig::default()
    );

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
