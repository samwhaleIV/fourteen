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

use winit::event_loop::{
    ControlFlow,
    DeviceEvents,
    EventLoop
};

use crate::desktop_app::{
    DesktopApp,
    WindowEventTraceConfig
};

impl WindowEventTraceConfig for PlaceholderConfig {
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

pub fn main() -> anyhow::Result<()> {
    let log_variable = "RUST_LOG";

    match env::var(log_variable) {
        Ok(value) => println!("{}: {:?}",log_variable,value),
        Err(error) => println!("Error {}: {}",log_variable,error),
    }

    let mut builder = Builder::from_default_env();
    builder.target(Target::Stdout);
    builder.init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let wimpy_app = PlaceholderApp {} ;

    let mut desktop_app = DesktopApp::<PlaceholderApp,PlaceholderConfig>::new(wimpy_app);
    event_loop.listen_device_events(DeviceEvents::Always);

    log::info!("Starting event loop! Here we go. No going back now.");

    event_loop.run_app(&mut desktop_app)?;
    return Ok(());
}
