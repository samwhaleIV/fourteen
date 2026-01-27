const WINDOW_TITLE: &'static str = "Fourteen Engine - Hello, World!";
const MINIMUM_WINDOW_SIZE: (u32,u32) = (600,400);

use std::{cmp::Ordering, collections::{BTreeMap, BTreeSet, HashMap}, sync::Arc};

use image::{
    DynamicImage,
    ImageError,
    ImageReader
};

use sdl2::{controller::{Axis, Button, GameController}, event::{Event, WindowEvent}};
use wgpu::{Color, Limits};

use wimpy_engine::{
    WimpyApp,
    WimpyIO,
    WimpyImageError,
    input::{
        GamepadButtonSet, GamepadButtons, GamepadInput, GamepadJoystick, InputManager, InputManagerAppController, InputManagerReadonly
    },
    wgpu::{
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

pub trait WindowEventTraceConfig {
    const LOG_REDRAW: bool;
    const LOG_MOUSE_MOVE: bool;
    const LOG_WINDOW_MOVE: bool;
    const LOG_RESIZE: bool;
    const LOG_MOUSE_OVER_WINDOW: bool;
    const LOG_MOUSE_CLICK: bool;
    const KEY_CHANGE: bool;
    const LOG_WINDOW_FOCUS: bool;
    const LOG_OTHER: bool;
}

struct DekstopAppIO;

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

pub fn run_desktop_app<TWimpyApp,TConfig>(wimpy_app: TWimpyApp)
where
    TConfig: GraphicsContextConfig
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

    let mut graphics_provider = match pollster::block_on(GraphicsProvider::new(GraphicsProviderConfig {
        instance,
        surface,
        limits: Limits::defaults(),
    })) {
        Ok(device) => device,
        Err(error) => {
            log::error!("Failure to initialize wgpu: {:?}",error);
            todo!();
        }
    };

    let window_size = window.size();
    graphics_provider.set_size(window_size.0,window_size.1);

    let mut event_pump = sdl_context.event_pump().expect("sdl event pump creation");

    let mut graphics_context = GraphicsContext::<TConfig>::create(graphics_provider);
    let mut input_manager = InputManager::default();

    let mut active_controller: Option<(u32,GameController)> = None;
    let mut loaded_controllers: BTreeMap<u32,GameController> = Default::default();

    'event_loop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Window {
                    window_id,
                    win_event: WindowEvent::SizeChanged(width, height),
                    ..
                } if window_id == window.id() => {
                    graphics_context.get_graphics_provider_mut().set_size(
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
                        input_manager.set_key_code_pressed(wk);
                    }
                },
                Event::KeyUp {
                    keycode: Some(keycode),
                    repeat,
                    ..
                } => {
                    if !repeat && let Some(wk) = translate_key_code(keycode) {
                        input_manager.set_key_code_released(wk);
                    }
                },
                Event::Quit { .. } => {
                    break 'event_loop;
                },
                Event::ControllerDeviceAdded { which, .. } => {
                    if let Some(controller_system) = &game_controller_subsystem {
                        match controller_system.open(which) {
                            Ok(controller) => {
                                let id = controller.instance_id();
                                log::info!(
                                    "Controller device added with ID '{}' (UUID: {:?}).",
                                    controller.instance_id(),
                                    controller.product_id()
                                );
                                if active_controller.is_none() {
                                    log::info!(
                                        "Active controller set to ID '{}' (New Device).",
                                        controller.instance_id()
                                    );
                                    active_controller = Some((id,controller));
                                } else {
                                    loaded_controllers.insert(id,controller);
                                }
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
                    if let Some(controller) = &active_controller && controller.0 == which {
                        active_controller = None;
                        if let Some(fallback_controller) = loaded_controllers.pop_first() {
                            log::info!(
                                "Active controller set to ID '{}' (Existing Device).",
                                fallback_controller.0
                            );
                            active_controller = Some(fallback_controller);
                        } else {
                            log::warn!("There is are no controllers left to swap the active controller.");
                        }
                    } else {
                        _ = loaded_controllers.remove(&which);
                    }
                }
                _ => {}
            }
        }

        let gamepad_state = match &active_controller {
            Some((_,controller)) => get_gamepad_state(controller),
            None => Default::default(),
        };

        //log::info!("Left Stick: {:?}",gamepad_state.left_trigger);

        input_manager.update(gamepad_state);

        

        let mut output_frame = match graphics_context.create_output_frame(Color::RED) {
            Ok(value) => value,
            Err(error) => {
                log::error!("Could not create output frame: {:?}",error);
                return;
            }
        };

        //TODO: interface wimpy_app

        if let Err(error) = graphics_context.render_frame(&mut output_frame) {
            log::error!("Could not render ouput frame: {:?}",error);
        }

        if let Err(error) = graphics_context.present_output_frame(output_frame) {
            log::error!("Could not present output frame: {:?}",error);
        }
    }
}

fn get_gamepad_state(controller: &GameController) -> GamepadInput {
    GamepadInput {
        buttons: GamepadButtons::from_set(GamepadButtonSet {
            dpad_up: controller.button(Button::DPadUp),
            dpad_down: controller.button(Button::DPadDown),
            dpad_left: controller.button(Button::DPadLeft),
            dpad_right: controller.button(Button::DPadRight),

            select: controller.button(Button::Back),
            start: controller.button(Button::Start),

            a: controller.button(Button::A),
            b: controller.button(Button::B),
            x: controller.button(Button::X),
            y: controller.button(Button::Y),
    

            left_bumper: controller.button(Button::LeftShoulder),
            right_bumper: controller.button(Button::RightShoulder),

            left_stick: controller.button(Button::LeftStick),
            right_stick: controller.button(Button::RightStick),
        }),
        left_stick: GamepadJoystick {
            x: cast_axis_value(controller.axis(Axis::LeftX)),
            y: cast_axis_value(controller.axis(Axis::LeftY)),
        },
        right_stick: GamepadJoystick {
            x: cast_axis_value(controller.axis(Axis::RightX)),
            y: cast_axis_value(controller.axis(Axis::RightY))
        },
        left_trigger: cast_axis_value(controller.axis(Axis::TriggerLeft)),
        right_trigger: cast_axis_value(controller.axis(Axis::TriggerRight)),
    }
}

fn cast_axis_value(value: i16) -> f32 {
    value as f32 / i16::MAX as f32
}
