mod desktop_app;
mod desktop_device;

use std::env;

use wimpy_engine::{
    WimpyAppHandler, wgpu::GraphicsContextConfig
};

use env_logger::{
    Builder,
    Target
};

use winit::event_loop::{
    ControlFlow,
    DeviceEvents,
    EventLoop
};

use crate::desktop_app::{DesktopApp, WindowEventTraceConfig};

struct EngineConfig;

impl GraphicsContextConfig for EngineConfig {
    const INSTANCE_CAPACITY: usize = 256;
    const UNIFORM_CAPACITY: usize = 64;
    const CACHE_INSTANCES: usize = 0;
    const CACHE_SIZES: &[(u32,u32)] = &[];
}

impl WindowEventTraceConfig for EngineConfig {
    const LOG_REDRAW: bool = false;
    const LOG_MOUSE_MOVE: bool = false;
    const LOG_WINDOW_MOVE: bool = false;
    const LOG_RESIZE: bool = true;
    const LOG_MOUSE_OVER_WINDOW: bool = true;
    const LOG_MOUSE_CLICK: bool = true;
    const KEY_CHANGE: bool = true;
    const LOG_WINDOW_FOCUS: bool = true;
    const LOG_OTHER: bool = true;
}

struct PlaceholderApp { }
impl WimpyAppHandler for PlaceholderApp { }

pub fn main() -> anyhow::Result<()> {
    let log_variable = "RUST_LOG";

    match env::var(log_variable) {
        Ok(value) => println!("{}: {:?}", log_variable, value),
        Err(error) => println!("Error {}: {}", log_variable, error),
    }

    let mut builder = Builder::from_default_env();
    builder.target(Target::Stdout);
    builder.init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let wimpy_app = PlaceholderApp {};

    let mut desktop_app = DesktopApp::<PlaceholderApp,EngineConfig>::new(wimpy_app);
    event_loop.listen_device_events(DeviceEvents::Always);

    log::info!("Starting event loop! Here we go. No going back now.");

    event_loop.run_app(&mut desktop_app)?;
    return Ok(());
}
