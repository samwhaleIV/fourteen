const WINDOW_TITLE: &'static str = "Twelve Engine - Hello, World!";
const MINIMUM_WINDOW_SIZE: (u32,u32) = (800,600);
const MAX_STATE_LOAD_PASSES: u32 = 32;

use std::sync::Arc;

use super::{
    virtual_device::VirtualDevice,
    app_state::*
};

use crate::graphics::{
    GraphicsContext,
    GraphicsContextConfiguration
};

use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize, Position},
    event::*,
    event_loop::{ActiveEventLoop},
    keyboard::{KeyCode,PhysicalKey},
    window::{Window,WindowId}
};

struct LongLifetimeHandles {
    window: Arc<Window>,
    device: VirtualDevice,
    context: GraphicsContext,
}

pub struct App {
    handles: Option<LongLifetimeHandles>,

    state_loaded: bool,
    surface_configured: bool,
    app_exiting: bool,

    frame_number: u128,
    event_number: u128,

    window_width: u32,
    window_height: u32,

    mouse_point: (f32,f32),
    state_generator: AppStateGenerator,
    state: AppState,

    log_trace_config: LogTraceConfig,
    context_options: Option<GraphicsContextConfiguration>,
}

enum EventLoopOperation {
    Continue,
    Terminate,
    Repeat
}

struct DummyAppState;

impl AppStateInterface for DummyAppState {
    /* Oh no! You've activated my trap card. */
    fn unload(&mut self,_device: &VirtualDevice,_context: &mut GraphicsContext) {
        panic!("Cannot unload the dummy state!");
    }
    
    fn update(&mut self) -> UpdateResult {
        panic!("Cannot update the dummy state!");
    }
    
    fn render(&self,_device: &VirtualDevice,_context: &mut GraphicsContext) {
        panic!("Cannot render the dummy state!");
    }
    
    fn input(&mut self,_event: InputEvent) {
        panic!("Cannot input to the dummy state!");
    }
}

#[derive(Default)]
pub struct LogTraceConfig {
    pub redraw: bool,
    pub mouse_move: bool,
    pub window_move: bool,
    pub resize: bool,
    pub mouse_over_window: bool,
    pub mouse_click: bool,
    pub key_change: bool,
    pub window_focus: bool,
    pub other: bool
}

fn placeholder_state_generator(_device: &VirtualDevice,_pipeline: &mut GraphicsContext) -> AppState {
    panic!("Cannot generate an AppState using the placeholder state generator");
}

pub struct AppConfiguration {
    pub state_generator: AppStateGenerator,
    pub context_options: Option<GraphicsContextConfiguration>,
    pub log_trace_config: Option<LogTraceConfig>,
}

fn get_center_position(parent: PhysicalSize<u32>,child: PhysicalSize<u32>) -> Position {
    let x = (parent.width - child.width) / 2;
    let y = (parent.height - child.height) / 2;
    return Position::Physical(PhysicalPosition::new(x as i32,y as i32));
}

impl App {
    pub fn create(options: AppConfiguration) -> Self {
        return Self {
            handles: None,

            state_generator: options.state_generator,
            state: Box::new(DummyAppState),

            state_loaded: false,
            surface_configured: false,

            app_exiting: false,

            context_options: options.context_options,

            window_width: 0,
            window_height: 0,

            frame_number: 0,
            event_number: 0,
            
            mouse_point: (0.0,0.0),
    
            /* For Debugging */
            log_trace_config: match options.log_trace_config {
                Some(value) => value,
                None => Default::default(),
            }
        }
    }

    fn try_configure_surface(&mut self) -> bool {
        let width = self.window_width;
        let height = self.window_height;

        if width < 1 || height < 1 {
            log::warn!("Cannot configure surface with one or more of the following size components: {}x{}",width,height);
            return false;
        }

        if let Some(handles) = &mut self.handles {
            handles.device.configure_surface_size(width,height);
        } else {
            let error = "Graphics object does not exist.";
            log::error!("{}",&error);
            panic!("{}",&error);
        }

        self.surface_configured = true;
        return true;
    }

    fn update(&mut self) -> EventLoopOperation {
        if !self.state_loaded {
            self.load_state();
        }
        let update_result = self.state.update();
        return match update_result.get_operation() {
            AppStateOperation::Continue => EventLoopOperation::Continue,

            AppStateOperation::Terminate => EventLoopOperation::Terminate,

            AppStateOperation::Transition => {
                if let Some(state_generator) = update_result.get_state_generator() {
                    self.unload_state();
                    self.state_generator = state_generator;
                    return EventLoopOperation::Repeat;
                } else {
                    log::error!("Invalid app state transition data.");
                    return EventLoopOperation::Terminate;
                }
            }
        };
    }

    /* Primary function to handle updating */
    fn handle_redraw(&mut self,event_loop: &ActiveEventLoop) {
        if self.log_trace_config.redraw {
            log::trace!("handle_redraw - frame_number:{} | event_number:{}",self.frame_number,self.event_number);
        }

        let mut load_attempts = 0;

        loop {

            if load_attempts >= MAX_STATE_LOAD_PASSES {
                self.terminate_app(event_loop);
                return; 
            }

            load_attempts += 1;

            match self.update() {
                EventLoopOperation::Continue => break,
                EventLoopOperation::Terminate => {
                    self.terminate_app(event_loop);
                    return;
                },
                EventLoopOperation::Repeat => continue
            }
        }

        if let Some(mut handles) = self.handles.take() {
            if !self.state_loaded {
                log::warn!("Attempt to render without a loaded state.");
            }
            if self.state_loaded && (self.surface_configured || self.try_configure_surface()) {
                self.state.render(&handles.device,&mut handles.context);
            }
            handles.window.request_redraw();
            self.handles = Some(handles);
        } else {
            log::error!("Window handles not found.");
            self.terminate_app(event_loop);
        }

        self.frame_number += 1;
    }

    fn load_state(&mut self) {
        if self.state_loaded {
            log::warn!("Cannot load state, we are already in a loaded state.");
            return;
        }

        if let Some(mut handles) = self.handles.take() {
            let new_state = (self.state_generator)(&handles.device,&mut handles.context);
            self.state_generator = placeholder_state_generator;
            self.state = new_state;
            self.handles = Some(handles);
        }

        self.state_loaded = true;
    }

    fn unload_state(&mut self) {
        if !self.state_loaded {
            if !self.app_exiting {
                log::warn!("Cannot unload state, we are already in an unloaded state.");
            }
            return;
        }
        if let Some(mut handles) = self.handles.take() {
            self.state.unload(&handles.device,&mut handles.context);
            self.handles = Some(handles);    
        } else if !self.app_exiting {
            log::warn!("Unusual app exit state. The app handles have already been dropped.");
            return;
        }
        self.state = Box::new(DummyAppState);
        self.state_loaded = false;
    }

    fn terminate_app(&mut self,event_loop: &ActiveEventLoop) {
        if self.app_exiting {
            log::warn!("App termination is already marked.");
            return;
        }
        log::info!("Terminating app; it's the right thing to do.");
        self.app_exiting = true;
        self.unload_state();
        event_loop.exit();
        log::info!("Termination success. Event loop exiting.");
    }

    fn handle_key_change(&mut self,_code: KeyCode,_pressed: bool){
        if self.log_trace_config.key_change {
            log::trace!("handle_key_change - frame_number:{} | event_number:{}",self.frame_number,self.event_number);
        }
        //TODO
    }

    fn handle_mouse_press(&mut self) {
        if self.log_trace_config.mouse_click {
            log::trace!("handle_mouse_press - frame_number:{} | event_number:{}",self.frame_number,self.event_number);
        }
        if !self.state_loaded {
            return;
        }
        //TODO
    }

    fn handle_mouse_release(&mut self) {
        if self.log_trace_config.mouse_click {
            log::trace!("handle_mouse_release - frame_number:{} | event_number:{}",self.frame_number,self.event_number);
        }
        if !self.state_loaded {
            return;
        }
        //TODO
    }

    fn handle_mouse_move(&mut self,point: (f32,f32)) {
        if self.log_trace_config.mouse_move {
            log::trace!("handle_mouse_move - frame_number:{} | event_number:{} | x:{} y:{}",self.frame_number,self.event_number,point.0,point.1);
        }
        self.mouse_point = point;
        if !self.state_loaded {
            return;
        }
        //TODO
    }
}

impl ApplicationHandler for App {

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {

        if self.handles.is_some() {
            /* This shouldn't happen on desktop platforms. */
            log::info!("The app has been resumed. Welcome back.");
            return;
        } else {
            log::info!("Received 'resumed' call. Getting on with the window and graphics setup.");
        }
        
        let (min_width,min_height) = MINIMUM_WINDOW_SIZE;
        let min_inner_size = PhysicalSize::new(min_width,min_height);
        let window_size = PhysicalSize::new(min_width,min_height); /* TODO: Load last window size of application. */

        let window_attributes = Window::default_attributes()
            .with_title(WINDOW_TITLE)
            .with_min_inner_size(min_inner_size)
            .with_inner_size(window_size)
            .with_visible(false);

        let window = Arc::new(match event_loop.create_window(window_attributes) {
            Ok(window) => window,
            Err(error) => {
                log::error!("Could not create window through event loop: {}",error);
                self.terminate_app(event_loop);
                return;
            }
        });

        /* It's okay that this is optional; it's only used to center the window. */
        if let Some(monitor) = window.current_monitor() {
            let position = get_center_position(monitor.size(),window.outer_size());
            window.set_outer_position(position);
        }

        let device = match pollster::block_on(VirtualDevice::new(window.clone())) {
            Ok(device) => device,
            Err(error) => {
                log::error!("{}",error);
                self.terminate_app(event_loop);
                return;
            }
        };
        window.set_visible(true);

        let context = GraphicsContext::create(&device,match self.context_options.take() { /* We take so the underlying vectors (if any) are dropped. */
            Some(options) => options,
            None => GraphicsContextConfiguration::default(),
        });

        self.handles = Some(LongLifetimeHandles {
            window,
            device,
            context
        });

        log::info!("Graphics, pipeline, and window are now configured.");
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        /* This shouldn't happen on desktop platforms. */
        log::info!("App suspended. Goodnight, sweet prince.");
    }

    fn device_event(&mut self,_event_loop: &ActiveEventLoop,_device_id: DeviceId,event: DeviceEvent) {
        if self.app_exiting || !self.state_loaded {
            return;
        }
        match event {
            DeviceEvent::MouseMotion { delta } => self.state.input(InputEvent::MouseMoveDelta((delta.0 as f32,delta.1 as f32))),
            _ => {}
        }
    }

    fn window_event(&mut self,event_loop: &ActiveEventLoop,_window_id: WindowId,event: WindowEvent) {
        if self.app_exiting {
            return;
        }
        match event {
            WindowEvent::RedrawRequested => self.handle_redraw(event_loop),

            WindowEvent::Resized(size) => {    
                if self.log_trace_config.resize {
                   log::trace!("resized - frame_number:{} | event_number:{} | {}x{}",self.frame_number,self.event_number,size.width,size.height);
                }

                self.window_width = size.width;
                self.window_height = size.height;

                self.try_configure_surface();
            },

            WindowEvent::KeyboardInput {
                is_synthetic: false,
                event: KeyEvent {
                    physical_key: PhysicalKey::Code(code),
                    state: key_state,
                    repeat: false,
                    ..
                },
                device_id: _
            } => {
                let key_pressed = key_state == ElementState::Pressed;
                self.handle_key_change(code,key_pressed);
            },

            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                device_id: _
            } => match state {
                ElementState::Pressed => self.handle_mouse_press(),
                ElementState::Released => self.handle_mouse_release(),
            },

            WindowEvent::CursorMoved { position, device_id: _ } => {
                self.handle_mouse_move((position.x as f32,position.y as f32));
            }

            WindowEvent::CursorEntered { device_id: _ } => {
                if self.log_trace_config.mouse_over_window {
                    log::trace!("handle_mouse_enter - frame_number:{} | event_number:{}",self.frame_number,self.event_number);
                }
            },

            WindowEvent::CursorLeft { device_id: _ } => {
                if self.log_trace_config.mouse_over_window {
                    log::trace!("handle_mouse_leave - frame_number:{} | event_number:{}",self.frame_number,self.event_number);
                }
            },

            WindowEvent::Moved(_) => {
                if self.log_trace_config.window_move {
                    log::trace!("window moved - frame_number:{} | event_number:{}",self.frame_number,self.event_number);
                }
            },

            WindowEvent::Focused(focused) => match focused {
                true => {
                    if self.log_trace_config.window_focus {
                        log::trace!("window focused - frame_number:{} | event_number:{}",self.frame_number,self.event_number);
                    }
                },

                false => {
                    if self.log_trace_config.window_focus {
                        log::trace!("window lost focus - frame_number:{} | event_number:{}",self.frame_number,self.event_number);
                    }
                },
            },

            /* TODO: Might want to use this ? */
            WindowEvent::ScaleFactorChanged { .. } => {
                if self.log_trace_config.other {
                    log::trace!("scale factor changed - frame_number:{} | event_number:{}",self.frame_number,self.event_number);
                }
            },

            WindowEvent::CloseRequested | WindowEvent::Destroyed => self.terminate_app(event_loop), //Goodbye, cruel world.

            _ => {}
        };

        self.event_number += 1;
    }
}
