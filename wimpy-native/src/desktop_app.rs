const WINDOW_TITLE: &'static str = "Wimpy Engine";
const MINIMUM_WINDOW_SIZE: (u32,u32) = (600,400);

use std::{
    collections::HashMap,
    path::Path
};

use sdl2::{
    EventPump, GameControllerSubsystem, Sdl, VideoSubsystem, controller::{
        Axis,
        Button,
        GameController
    }, event::{
        Event,
        WindowEvent
    }
};

use wgpu::{Instance, Limits, Surface};

use wimpy_engine::app::*;
use wimpy_engine::app::wam::*;
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
    wimpy_app: TWimpyApp,
    active_gamepad: Option<GameController>,
    unused_gamepads: HashMap<u32,GameController>,
    graphics_context: GraphicsContext,
    input_manager: InputManager,
    asset_manager: AssetManager,
    window_id: u32,
    key_value_store: KeyValueStore,
}

struct SDLSystems {
    main: Sdl,
    video: VideoSubsystem,
    game_controller: Option<GameControllerSubsystem>,
}

async fn async_load<TWimpyApp,TConfig>(
    manifest_path: Option<&Path>,
    instance: Instance,
    surface: Surface<'static>,
    window_id: u32,
    window_size: (u32,u32),
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
    graphics_provider.set_size(window_size.0,window_size.1);

    let mut asset_manager = AssetManager::load_or_default::<DekstopAppIO>(manifest_path).await;

    let mut graphics_context = GraphicsContext::create::<TConfig>(graphics_provider);
    let mut key_value_store = KeyValueStore::default();

    let mut input_manager = InputManager::with_input_type_hint(
        InputType::Unknown
        // Reminder: Set input type ahead of time on specific platforms.
    );

    let wimpy_app = TWimpyApp::load(&mut WimpyContext {
        graphics: &mut graphics_context,
        storage: &mut key_value_store,
        input: &mut input_manager,
        assets: &mut asset_manager
    }).await;

    return Some(InnerApp {
        sdl: sdl_systems,
        wimpy_app,
        active_gamepad: None,
        unused_gamepads: Default::default(),
        graphics_context,
        input_manager,
        asset_manager,
        window_id,
        key_value_store,
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

    let sdl_systems = SDLSystems {
        game_controller: game_controller_subsystem,
        video: video_subsystem,
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
        surface,window.id(),
        window.size(),
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
        let gamepad_state = match &self.active_gamepad {
            Some(gamepad) => get_gamepad_state(gamepad),
            None => Default::default(),
        };

        self.input_manager.update(gamepad_state);

        self.wimpy_app.update(&mut WimpyContext {
            graphics: &mut self.graphics_context,
            storage: &mut self.key_value_store,
            input: &mut self.input_manager,
            assets: &mut self.asset_manager
        });
    }

    fn poll_events(&mut self,event_pump: &mut EventPump) -> EventLoopOperation {
        for event in event_pump.poll_iter() {
            match event {
                Event::Window {
                    window_id,
                    win_event: WindowEvent::SizeChanged(width, height),
                    ..
                } if window_id == self.window_id => {
                    self.graphics_context.get_graphics_provider_mut().set_size(
                        width as u32,
                        height as u32
                    );
                }
                Event::KeyDown {
                    keycode: Some(keycode),
                    repeat,
                    ..
                } => {
                    if !repeat && let Some(wk) = translate_key_code(keycode) {
                        self.input_manager.set_key_code_pressed(wk);
                    }
                },
                Event::KeyUp {
                    keycode: Some(keycode),
                    repeat,
                    ..
                } => {
                    if !repeat && let Some(wk) = translate_key_code(keycode) {
                        self.input_manager.set_key_code_released(wk);
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
                Event::ControllerAxisMotion { which, value, axis, .. } => {
                    if self.active_gamepad.is_some() {
                        continue;
                    }
                    let axis_value = cast_axis_value(value);
                    if match axis {
                        Axis::TriggerLeft | Axis::TriggerRight => significant_trigger_difference(
                            0.0,
                            axis_value
                        ),
                        Axis::LeftX | Axis::LeftY | Axis::RightX | Axis::RightY => significant_axis_difference(
                            0.0,
                            axis_value
                        ),
                    } {
                        self.set_active_controller(which);
                    }
                },
                Event::ControllerButtonDown { which, .. } |
                Event::ControllerButtonUp { which, .. } |
                Event::ControllerTouchpadDown { which, .. } |
                Event::ControllerTouchpadUp { which, .. } |
                Event::ControllerTouchpadMotion { which, .. } => {
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
