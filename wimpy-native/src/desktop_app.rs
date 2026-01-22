const WINDOW_TITLE: &'static str = "Fourteen Engine - Hello, World!";
const MINIMUM_WINDOW_SIZE: (u32,u32) = (600,400);

use std::sync::Arc;

use wgpu::Limits;
use wimpy_engine::{WimpyApp, WimpyIO, input::{InputManager, InputManagerAppController}, wgpu::{GraphicsContext, GraphicsContextConfig, GraphicsContextController, GraphicsContextInternalController, GraphicsProvider, GraphicsProviderConfig}};
use winit::{
    application::ApplicationHandler,
    dpi::{
        PhysicalPosition,
        PhysicalSize,
        Position
    },
    event::{self, *},
    event_loop::ActiveEventLoop,
    keyboard::PhysicalKey,
    window::{
        Window,
        WindowId,
    }
};

use crate::key_code::translate_key_code;

pub struct DesktopApp<TWimpyApp,TConfig> {
    window: Option<Arc<Window>>,
    graphics_context: Option<GraphicsContext<TConfig>>,
    input_manager: InputManager,
    wimpy_app: TWimpyApp,
    frame_number: u128,
    event_number: u128,
}

impl<TWimpyApp,TConfig> DesktopApp<TWimpyApp,TConfig> {
    pub fn new(wimpy_app: TWimpyApp) -> Self {
        return Self {
            window: None,
            graphics_context: None,
            wimpy_app,
            frame_number: Default::default(),
            event_number: Default::default(),
            input_manager: Default::default(),
        }
    }
}

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

fn get_center_position(parent: PhysicalSize<u32>,child: PhysicalSize<u32>) -> Position {
    let x = (parent.width - child.width) / 2;
    let y = (parent.height - child.height) / 2;
    return Position::Physical(PhysicalPosition::new(x as i32,y as i32));
}

struct DekstopAppIO;

impl WimpyIO for DekstopAppIO {
    fn save_key_value_store(kvs: &wimpy_engine::storage::KeyValueStore) {
        todo!()
    }

    fn load_key_value_store(kvs: &mut wimpy_engine::storage::KeyValueStore) {
        todo!()
    }

    fn get_file_bytes(file: &'static str) -> Result<Vec<u8>,wimpy_engine::WimpyIOError> {
        todo!()
    }
}

impl<TWimpyApp,TConfig> ApplicationHandler for DesktopApp<TWimpyApp,TConfig>
where
    TConfig: GraphicsContextConfig + WindowEventTraceConfig,
    TWimpyApp: WimpyApp<DekstopAppIO,TConfig>,
{
    fn resumed(&mut self,event_loop: &ActiveEventLoop) {
        if self.graphics_context.is_some() {
            return;
        };
        
        let (min_width,min_height) = MINIMUM_WINDOW_SIZE;
        let min_inner_size = PhysicalSize::new(min_width,min_height);
        let window_size = PhysicalSize::new(min_width,min_height);

        let window_attributes = Window::default_attributes()
            .with_title(WINDOW_TITLE)
            .with_min_inner_size(min_inner_size)
            .with_inner_size(window_size)
            .with_visible(false);

        let window = Arc::new(event_loop.create_window(window_attributes).expect("window creation"));

        if let Some(monitor) = window.current_monitor() {
            let position = get_center_position(monitor.size(),window.outer_size());
            window.set_outer_position(position);
        }

        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).expect("surface creation");

        self.window = Some(window.clone());

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
        graphics_provider.set_size(size.width,size.height);

        self.graphics_context = Some(GraphicsContext::create(graphics_provider));

        window.set_visible(true);
        log::info!("App graphics context and shared state are configured.");
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        log::info!("Application suspended. This shouldn't happen on desktop platforms.");
    }

    fn device_event(&mut self,_event_loop: &ActiveEventLoop,_device_id: DeviceId,event: DeviceEvent) {
        return;
        #[allow(unused)]
        match event {
            DeviceEvent::Added => todo!(),
            DeviceEvent::Removed => todo!(),
            DeviceEvent::MouseMotion { delta } => todo!(),
            DeviceEvent::MouseWheel { delta } => todo!(),
            DeviceEvent::Motion { axis, value } => todo!(),
            DeviceEvent::Button { button, state } => todo!(),
            DeviceEvent::Key(raw_key_event) => todo!(),
        }
    }

    fn window_event(&mut self,event_loop: &ActiveEventLoop,_window_id: WindowId,event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => {
                let Some(window) = &self.window else {
                    log::error!("Redraw requested, but window could not be located.");
                    return;
                };

                let Some(graphics_context) = &mut self.graphics_context else {
                    log::error!("Redraw requested, but graphics context could not be located.");
                    return;
                };

                let mut output_frame = match graphics_context.create_output_frame() {
                    Ok(value) => value,
                    Err(error) => {
                        log::error!("Could not create output frame: {:?}",error);
                        return;
                    }
                };

                //TODO.... stuff

                if let Err(error) = graphics_context.bake(&mut output_frame) {
                    log::error!("{:?}",error);
                }

                if let Err(error) = graphics_context.present_output_frame() {
                    log::error!("{:?}",error);
                }

                window.request_redraw();
            },

            WindowEvent::Resized(size) => {    
                if TConfig::LOG_RESIZE {
                   log::trace!("resized - frame_number:{} | event_number:{} | {}x{}",self.frame_number,self.event_number,size.width,size.height);
                }
                let Some(graphics_context) = &mut self.graphics_context else {
                    return;
                };
                graphics_context.get_graphics_provider_mut().set_size(size.width,size.height);
            },

            WindowEvent::KeyboardInput {
                is_synthetic: false,
                event: KeyEvent {
                    physical_key: PhysicalKey::Code(winit_key_code),
                    state: ElementState::Pressed,
                    repeat: false,
                    ..
                },
                device_id: _
            } => {
                if let Some(wimpy_key_code) = translate_key_code(winit_key_code) {
                    self.input_manager.set_key_code_pressed(wimpy_key_code);
                }
            },

            WindowEvent::KeyboardInput {
                is_synthetic: false,
                event: KeyEvent {
                    physical_key: PhysicalKey::Code(winit_key_code),
                    state: ElementState::Released,
                    repeat: false,
                    ..
                },
                device_id: _
            } => {
                if let Some(wimpy_key_code) = translate_key_code(winit_key_code) {
                    self.input_manager.set_key_code_released(wimpy_key_code);
                }
            },

            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state: ElementState::Pressed,
                device_id: _
            } => {
                // self.send_input(InputEvent::MousePress(self.mouse_point));
            },

            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state: ElementState::Released,
                device_id: _
            } => {
                // self.send_input(InputEvent::MousePress(self.mouse_point));
            },

            WindowEvent::CursorMoved { position, device_id: _ } => {
                // self.mouse_point = (position.x as f32,position.y as f32);
                // self.send_input(InputEvent::MouseMove(self.mouse_point));
            }

            WindowEvent::CursorEntered { device_id: _ } => {
                if TConfig::LOG_MOUSE_OVER_WINDOW {
                    log::trace!("handle_mouse_enter - frame_number:{} | event_number:{}",self.frame_number,self.event_number);
                }
            },

            WindowEvent::CursorLeft { device_id: _ } => {
                if TConfig::LOG_MOUSE_OVER_WINDOW {
                    log::trace!("handle_mouse_leave - frame_number:{} | event_number:{}",self.frame_number,self.event_number);
                }
            },

            WindowEvent::Moved(_) => {
                if TConfig::LOG_MOUSE_MOVE {
                    log::trace!("window moved - frame_number:{} | event_number:{}",self.frame_number,self.event_number);
                }
            },

            WindowEvent::Focused(focused) => match focused {
                true => {
                    if TConfig::LOG_WINDOW_FOCUS {
                        log::trace!("window focused - frame_number:{} | event_number:{}",self.frame_number,self.event_number);
                    }
                },
                false => {
                    if TConfig::LOG_WINDOW_FOCUS {
                        log::trace!("window lost focus - frame_number:{} | event_number:{}",self.frame_number,self.event_number);
                    }
                },
            },

            /* TODO: Might want to use this ? */
            WindowEvent::ScaleFactorChanged { .. } => {
                if TConfig::LOG_OTHER {
                    log::trace!("scale factor changed - frame_number:{} | event_number:{}",self.frame_number,self.event_number);
                }
            },

            WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                event_loop.exit();
            },

            _ => {}
        };

        self.event_number = self.event_number.wrapping_add(1);
    }
}
