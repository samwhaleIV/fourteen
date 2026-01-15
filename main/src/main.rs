mod test_state;

use env_logger::{
    Builder,
    Target
};

use std::env;
use wimpy::{shared::CacheArenaConfig, wgpu::GraphicsContextConfig};

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

struct EngineConfig;

impl GraphicsContextConfig for EngineConfig {
    const INSTANCE_CAPACITY: usize = 256;
    const UNIFORM_CAPACITY: usize = 64;
    const CACHE_INSTANCES: usize = 0;
    const CACHE_SIZES: &[(u32,u32)] = &[];
}

fn create_event_loop() -> anyhow::Result<()> {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::create(AppConfiguration::<SharedState,TConfig> {
        state_generator: generate_test_state,
        shared_state_generator: SharedState::generator,
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
