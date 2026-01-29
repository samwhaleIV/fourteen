const WINDOW_TITLE: &'static str = "Fourteen Engine - Hello, World!";
const MINIMUM_WINDOW_SIZE: (u32,u32) = (600,400);

use std::collections::HashMap;

use image::{
    DynamicImage,
    ImageError,
    ImageReader
};

use sdl2::{
    EventPump,
    GameControllerSubsystem,
    Sdl,
    controller::{
        Axis,
        Button,
        GameController
    },
    event::{
        Event,
        WindowEvent
    },
    sys::Window
};

use wgpu::{
    Color,
    Limits
};

use wimpy_engine::{
    WimpyApp,
    WimpyContext,
    WimpyIO,
    WimpyImageError,
    input::{
        GamepadButtonSet,
        GamepadButtons,
        GamepadInput,
        GamepadJoystick,
        InputManager,
        InputManagerAppController,
        InputManagerReadonly,
        InputType,
        significant_axis_difference,
        significant_trigger_difference
    },
    storage::KeyValueStore, wgpu::{
        GraphicsContext,
        GraphicsContextConfig,
        GraphicsContextController,
        GraphicsContextInternalController,
        GraphicsProvider,
        GraphicsProviderConfig,
        TextureData,
        TextureDataWriteParameters
    }
};

use crate::key_code::translate_key_code;

struct DynamicImageWrapper {
    value: DynamicImage
}

impl TextureData for DynamicImageWrapper {

    fn size(&self) -> (u32,u32) {
        (self.value.width(),self.value.height())
    }
    
    fn write_to_queue(self,parameters: &TextureDataWriteParameters) {
        parameters.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: parameters.texture,
                mip_level: parameters.mip_level,
                origin: parameters.origin,
                aspect: parameters.aspect,
            },
            self.value.as_bytes(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                /* 1 byte per color in 8bit 4 channel color (RGBA with u8) */
                bytes_per_row: Some(self.value.width() * 4), 
                rows_per_image: Some(self.value.height()),
            },
            parameters.texture_size,
        );
    }
}

pub struct DekstopAppIO;

impl WimpyIO for DekstopAppIO {
    fn save_key_value_store(kvs: &wimpy_engine::storage::KeyValueStore) {
        todo!()
    }

    fn load_key_value_store(kvs: &mut wimpy_engine::storage::KeyValueStore) {
        todo!()
    }

    async fn get_image(path: &'static str) -> Result<impl TextureData,WimpyImageError> {
        match ImageReader::open(path) {
            Ok(image_reader) => match image_reader.decode() {
                Ok(value) => Ok(DynamicImageWrapper { value }),
                Err(image_error) => Err(match image_error {
                    ImageError::Decoding(decoding_error) => {
                        log::error!("Image decode error: {:?}",decoding_error);
                        WimpyImageError::Decode
                    },
                    ImageError::Unsupported(unsupported_error) => {
                        log::error!("Image unsupported error: {:?}",unsupported_error);
                        WimpyImageError::UnsupportedFormat
                    },
                    ImageError::IoError(error) => {
                        log::error!("Image IO error: {:?}",error);
                        WimpyImageError::Access
                    },
                    _ => WimpyImageError::Unknown
                }),
            },
            Err(error) => Err({
                log::error!("IO error: {:?}",error);
                WimpyImageError::Access
            }),
        }
    }
}

enum EventLoopOperation {
    Continue,
    Terminate
}

struct InnerApp<TWimpyApp,TConfig> {
    wimpy_app: TWimpyApp,
    active_gamepad: Option<GameController>,
    unused_gamepads: HashMap<u32,GameController>,
    graphics_context: GraphicsContext<TConfig>,
    game_controller_subsystem: Option<GameControllerSubsystem>,
    input_manager: InputManager,
    window_id: u32,
    kvs_store: KeyValueStore,
    sdl_context: Sdl,
}

async fn async_load<TWimpyApp,TConfig>(mut wimpy_app: TWimpyApp) -> Option<InnerApp<TWimpyApp,TConfig>>
where
    TConfig: GraphicsContextConfig,
    TWimpyApp: WimpyApp<DekstopAppIO,TConfig>
{
    let sdl_context = sdl2::init().expect("sdl context creation");
    let video_subsystem = sdl_context.video().expect("sdl video subsystem creation");

    let game_controller_subsystem = match sdl_context.game_controller() {
        Ok(value) => Some(value),
        Err(error) => {
            log::error!("Could not initialize game controller subsystem: {}",error);
            None
        },
    };

    let window = video_subsystem
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

    let mut graphics_context = GraphicsContext::<TConfig>::create(graphics_provider);
    let mut kvs_store = KeyValueStore::default();

    let mut input_manager = InputManager::with_input_type_hint(
        InputType::Unknown
        // Reminder: Set input type ahead of time on specific platforms.
    );

    if let Err(error) = wimpy_app.load(&WimpyContext {
        graphics: &mut graphics_context,
        storage: &mut kvs_store,
        input: &mut input_manager
    }).await {
        log::error!("Failure to load wimpy add: {:?}",error);
        return None;
    }

    Some(InnerApp {
        sdl_context,
        active_gamepad: None,
        wimpy_app,
        unused_gamepads: Default::default(),
        graphics_context,
        game_controller_subsystem,
        window_id: window.id(),
        input_manager,
        kvs_store
    })
}

pub fn run_desktop_app<TWimpyApp,TConfig>(wimpy_app: TWimpyApp)
where
    TConfig: GraphicsContextConfig,
    TWimpyApp: WimpyApp<DekstopAppIO,TConfig>
{
    if let Some(mut inner_app) = pollster::block_on(async_load(wimpy_app)) {
        inner_app.start_loop();
    }
}

impl<TWimpyApp,TConfig> InnerApp<TWimpyApp,TConfig>
where
    TConfig: GraphicsContextConfig,
    TWimpyApp: WimpyApp<DekstopAppIO,TConfig>
{
    fn start_loop(&mut self) {
        let mut event_pump = self.sdl_context.event_pump().expect("sdl event pump creation");
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
        log::info!("Axes: {:?}",self.input_manager.get_axes().get_f32());

        let mut output_frame = match self.graphics_context.create_output_frame(Color::RED) {
            Ok(value) => value,
            Err(error) => {
                log::error!("Could not create output frame: {:?}",error);
                return;
            }
        };

        /* Update and render the user app! */
        self.wimpy_app.update(&WimpyContext {
            graphics: &mut self.graphics_context,
            storage: &mut self.kvs_store,
            input: &mut self.input_manager
        });

        if let Err(error) = self.graphics_context.render_frame(&mut output_frame) {
            log::error!("Could not render ouput frame: {:?}",error);
        }

        if let Err(error) = self.graphics_context.present_output_frame(output_frame) {
            log::error!("Could not present output frame: {:?}",error);
        }
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
                    if let Some(gamepad_system) = &self.game_controller_subsystem {
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
