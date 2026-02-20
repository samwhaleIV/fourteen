const WINDOW_TITLE: &'static str = "Wimpy Engine";
const MINIMUM_WINDOW_SIZE: (u32,u32) = (600,400);

const LEFT_EDGE_VIRTUAL_MODE_MARGIN: u32 = 2;
const RIGHT_EDGE_VIRTUAL_MODE_MARGIN: u32 = 8;

use std::{
    collections::HashMap,
    path::Path
};

use sdl2::{
    EventPump, GameControllerSubsystem, Sdl, TimerSubsystem, VideoSubsystem, controller::{
        Axis,
        Button,
        GameController
    }, event::{
        Event,
        WindowEvent
    }, mouse::MouseButton, video::Window
};

use wgpu::{Instance, Limits, Surface};

use wimpy_engine::{app::*, shared::WimpyArea};
use wimpy_engine::app::input::*;
use wimpy_engine::app::graphics::*;

use crate::{
    desktop_io::DekstopAppIO,
    key_code::translate_key_code
};

enum EventLoopOperation {
    Continue,
    Terminate
}

struct InnerApp<TWimpyApp> {
    sdl: SDLSystems,
    active_gamepad: Option<GameController>,
    unused_gamepads: HashMap<u32,GameController>,
    mouse_cache: MouseInput,
    window: Window,
    now: u64,
    wimpy_app: TWimpyApp,
    wimpy_context: WimpyContext,
}

struct SDLSystems {
    main: Sdl,
    timer: TimerSubsystem,
    video: VideoSubsystem,
    game_controller: Option<GameControllerSubsystem>,
}

async fn async_load<TWimpyApp,TConfig>(
    manifest_path: Option<&Path>,
    instance: Instance,
    surface: Surface<'static>,
    window: Window,
    sdl_systems: SDLSystems
) -> Option<InnerApp<TWimpyApp>>
where
    TConfig: GraphicsContextConfig,
    TWimpyApp: WimpyApp<DekstopAppIO>
{
    let mut graphics_provider = match GraphicsProvider::new(GraphicsProviderConfig {
        instance,
        surface,
        limits: Limits::defaults(),
    }).await {
        Ok(device) => device,
        Err(error) => {
            log::error!("Failure to initialize wgpu: {:?}",error);
            return None;
        }
    };
    let window_size = window.size();
    graphics_provider.set_size(window_size.0,window_size.1);

    let Some(mut wimpy_systems) = WimpyContext::create::<DekstopAppIO,TConfig>(WimpyContextCreationConfig {
        manifest_path,
        input_type_hint: InputType::Unknown,
        graphics_provider,
    }).await else {
        return None;
    };

    let wimpy_app = TWimpyApp::load(&mut wimpy_systems).await;

    let now = sdl_systems.timer.performance_counter();

    return Some(InnerApp {
        sdl: sdl_systems,
        active_gamepad: None,
        unused_gamepads: Default::default(),
        mouse_cache: Default::default(),
        window,
        now,
        wimpy_app,
        wimpy_context: wimpy_systems,
    });
}

pub fn run_desktop_app<TWimpyApp,TConfig>(manifest: Option<&Path>)
where
    TWimpyApp: WimpyApp<DekstopAppIO>,
    TConfig: GraphicsContextConfig
{
    let sdl = sdl2::init().expect("sdl context creation");
    let video_subsystem = sdl.video().expect("sdl video subsystem creation");

    let game_controller_subsystem = match sdl.game_controller() {
        Ok(value) => Some(value),
        Err(error) => {
            log::error!("Could not initialize game controller subsystem: {}",error);
            None
        },
    };

    let timer_subsystem = sdl.timer().expect("sdl timer subsystem");

    let sdl_systems = SDLSystems {
        game_controller: game_controller_subsystem,
        video: video_subsystem,
        timer: timer_subsystem,
        main: sdl
    };

    let window = sdl_systems.video
        .window(
            WINDOW_TITLE,
            MINIMUM_WINDOW_SIZE.0,
            MINIMUM_WINDOW_SIZE.1
        )
        .position_centered()
        .resizable()
        .metal_view()
        .build()
        .map_err(|e| e.to_string()).expect("sdl window creation");

    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::GL,
        ..Default::default()
    });

    let surface = unsafe {
        instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::from_window(&window).unwrap()).expect("sdl window surface creation")
    };

    if let Some(mut inner_app) = pollster::block_on(async_load::<TWimpyApp,TConfig>(
        manifest,
        instance,
        surface,
        window,
        sdl_systems
    )) {
        inner_app.start_loop();
    }
}

impl<TWimpyApp> InnerApp<TWimpyApp>
where
    TWimpyApp: WimpyApp<DekstopAppIO>
{
    fn start_loop(&mut self) {
        let mut event_pump = self.sdl.main.event_pump().expect("sdl event pump creation");
        'event_loop: loop {
            match self.poll_events(&mut event_pump) {
                EventLoopOperation::Continue => {
                    self.update();
                },
                EventLoopOperation::Terminate => {
                    break 'event_loop;
                },
            }
        }
    }

    fn update(&mut self) {
        let last = self.now;
        self.now = self.sdl.timer.performance_counter();

        let delta_seconds = ((self.now - last) as f64 / self.sdl.timer.performance_frequency() as f64) as f32;

        let gamepad_state = match &self.active_gamepad {
            Some(gamepad) => get_gamepad_state(gamepad),
            None => Default::default(),
        };

        let size = self.wimpy_context.graphics.get_graphics_provider().get_size();

        let shell_state = self.wimpy_context.input.update(
            self.mouse_cache,
            gamepad_state,
            delta_seconds,
            WimpyArea {
                x: LEFT_EDGE_VIRTUAL_MODE_MARGIN as f32,
                y: LEFT_EDGE_VIRTUAL_MODE_MARGIN as f32,
                width: (size.0 - RIGHT_EDGE_VIRTUAL_MODE_MARGIN) as f32,
                height: (size.1 - RIGHT_EDGE_VIRTUAL_MODE_MARGIN) as f32,
            },
            true
        );

        self.mouse_cache.delta = Default::default();

        let sdl_mouse = self.sdl.main.mouse();

        match shell_state.mouse_mode {
            MouseMode::Interface => {
                if sdl_mouse.relative_mouse_mode() {
                    sdl_mouse.set_relative_mouse_mode(false);
                }
            },
            MouseMode::Camera => {
                if !sdl_mouse.relative_mouse_mode() {
                    sdl_mouse.set_relative_mouse_mode(true);
                }
            },
        };

        if
            shell_state.should_reposition_hardware_cursor &&
            self.window.has_input_focus()
        {
            sdl_mouse.warp_mouse_in_window(
                &self.window,
                shell_state.position.x as i32, 
                shell_state.position.y as i32
            );
        }

        self.wimpy_app.update(&mut self.wimpy_context);
    }

    fn poll_events(&mut self,event_pump: &mut EventPump) -> EventLoopOperation {
        for event in event_pump.poll_iter() {
            match event {
                Event::Window {
                    window_id,
                    win_event: WindowEvent::FocusLost,
                    ..
                } if window_id == self.window.id() => {
                    let sdl_mouse = self.sdl.main.mouse();
                    if sdl_mouse.relative_mouse_mode() {
                        sdl_mouse.set_relative_mouse_mode(false);
                    }
                }
                Event::Window {
                    window_id,
                    win_event: WindowEvent::SizeChanged(width, height),
                    ..
                } if window_id == self.window.id() => {
                    self.wimpy_context.graphics.get_graphics_provider_mut().set_size(
                        width as u32,
                        height as u32
                    );
                },
                Event::MouseButtonDown { x, y, mouse_btn, .. } => {
                    self.mouse_cache.position = Position {
                        x: x as f32,
                        y: y as f32
                    };
                    match mouse_btn {
                        MouseButton::Left => {
                            self.mouse_cache.left_pressed = true;
                        },
                        MouseButton::Right => {
                            self.mouse_cache.right_pressed = true;
                        },
                        _ => {}
                    };
                },
                Event::MouseButtonUp { x, y, mouse_btn, .. } => {
                    self.mouse_cache.position = Position {
                        x: x as f32,
                        y: y as f32
                    };
                    match mouse_btn {
                        MouseButton::Left => {
                            self.mouse_cache.left_pressed = false;
                        },
                        MouseButton::Right => {
                            self.mouse_cache.right_pressed = false;
                        },
                        _ => {}
                    };
                },
                Event::MouseMotion { x, y, xrel, yrel, .. } => {
                    self.mouse_cache.delta.x += xrel as f32;
                    self.mouse_cache.delta.y += yrel as f32;
                    self.mouse_cache.position = Position {
                        x: x as f32,
                        y: y as f32
                    };
                },
                Event::KeyDown {
                    keycode: Some(keycode),
                    repeat,
                    ..
                } => {
                    if !repeat && let Some(wk) = translate_key_code(keycode) {
                        self.wimpy_context.input.set_key_code_pressed(wk);
                    }
                },
                Event::KeyUp {
                    keycode: Some(keycode),
                    repeat,
                    ..
                } => {
                    if !repeat && let Some(wk) = translate_key_code(keycode) {
                        self.wimpy_context.input.set_key_code_released(wk);
                    }
                },
                Event::Quit { .. } => {
                    return EventLoopOperation::Terminate;
                },
                Event::ControllerDeviceAdded { which, .. } => {
                    if let Some(gamepad_system) = &self.sdl.game_controller {
                        match gamepad_system.open(which) {
                            Ok(gamepad) => {
                                log::info!(
                                    "Controller device added with ID '{}' (UUID: {:?}).",
                                    gamepad.instance_id(),
                                    gamepad.product_id()
                                );
                                self.unused_gamepads.insert(which,gamepad);
                            },
                            Err(error) => {
                                log::error!(
                                    "Controller device error for ID '{}': {:?}",
                                    which,
                                    error
                                );
                            },
                        }
                    }
                },
                Event::ControllerDeviceRemoved { which, .. } => {
                    log::info!(
                        "Controller device disconnected for ID '{}'.",
                        which,
                    );
                    if let Some(controller) = &self.active_gamepad && controller.instance_id() == which {
                        self.active_gamepad = None;
                    } else {
                        if self.unused_gamepads.remove(&which).is_none() {
                            log::warn!("Untracked controller device removal");
                        }
                    }
                },
                Event::ControllerAxisMotion { which, .. } => {
                    if self.active_gamepad.is_none() {
                        self.set_active_controller(which);
                    }
                },
                Event::ControllerButtonDown { which, .. } |
                Event::ControllerButtonUp { which, .. } => {
                    if self.active_gamepad.is_none() {
                        self.set_active_controller(which);
                    }
                }
                _ => {}
            }
        }
        return EventLoopOperation::Continue;
    }
    fn set_active_controller(&mut self,which: u32) {

        match self.unused_gamepads.remove(&which) {
            Some(gamepad) => {
                self.active_gamepad = Some(gamepad);
                log::info!(
                    "Active controller set to ID '{}' (Activity was detected).",
                    which
                );
            },
            None => {
                log::warn!("Controller activity detected, this controller is untracked.");
            },
        }
    }
}

fn cast_axis_value(value: i16) -> f32 {
    value as f32 / i16::MAX as f32
}

fn trigger_clamp(value: f32) -> f32 {
    value.min(1.0).max(0.0)
}

fn get_gamepad_state(controller: &GameController) -> GamepadInput {
    GamepadInput {
        buttons: GamepadButtons::from_set(GamepadButtonSet {
            dpad_up:      controller.button(Button::DPadUp),
            dpad_down:    controller.button(Button::DPadDown),
            dpad_left:    controller.button(Button::DPadLeft),
            dpad_right:   controller.button(Button::DPadRight),

            select:       controller.button(Button::Back),
            start:        controller.button(Button::Start),
            guide:        controller.button(Button::Guide),

            a:            controller.button(Button::A),
            b:            controller.button(Button::B),
            x:            controller.button(Button::X),
            y:            controller.button(Button::Y),
    
            left_bumper:  controller.button(Button::LeftShoulder),
            right_bumper: controller.button(Button::RightShoulder),

            left_stick:   controller.button(Button::LeftStick),
            right_stick:  controller.button(Button::RightStick),
        }),
        left_stick: GamepadJoystick {
            x: cast_axis_value(
                controller.axis(Axis::LeftX)
            ),
            y: cast_axis_value(
                controller.axis(Axis::LeftY)
            ),
        },
        right_stick: GamepadJoystick {
            x: cast_axis_value(
                controller.axis(Axis::RightX)
            ),
            y: cast_axis_value(
                controller.axis(Axis::RightY)
            )
        },
        left_trigger: trigger_clamp(
            cast_axis_value(
                controller.axis(Axis::TriggerLeft)
            )
        ),
        right_trigger: trigger_clamp(
            cast_axis_value(
                controller.axis(Axis::TriggerRight)
            )
        ),
    }
}
