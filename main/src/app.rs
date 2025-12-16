const WINDOW_TITLE: &'static str = "Twelve Engine - Hello, World!";
const MINIMUM_WINDOW_SIZE: (u32,u32) = (800,600);

use std::sync::Arc;
use crate::app_state::*;
use crate::graphics::Graphics;
use wimpy::pipeline_management::{Pipeline, PipelineCreationOptions};

use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize, Position},
    event::*,
    event_loop::{ActiveEventLoop},
    keyboard::{KeyCode,PhysicalKey},
    window::{Window,WindowId}
};

struct PostConstructionHandles {
    window: Arc<Window>,
    graphics: Graphics,
    pipeline: Pipeline
}

pub struct App {
    handles: Option<PostConstructionHandles>,

    state_loaded: bool,
    surface_configured: bool,
    app_exiting: bool,

    frame_number: u128,
    event_number: u128,

    window_width: u32,
    window_height: u32,

    mouse_point: MousePoint,
    state_generator: AppStateGenerator,
    state: AppState,

    log_trace_config: LogTraceConfig,
    pipeline_options: Option<PipelineCreationOptions>,
}

enum EventLoopOperation {
    Continue,
    Terminate,
    Repeat
}

#[allow(dead_code)]
pub struct MouseDelta {
    pub x: f64,
    pub y: f64
}

struct DummyState;

impl AppStateHandler for DummyState {
    /* Oh no! You've activated my trap card. */
    fn unload(&mut self,_graphics: &Graphics,_pipeline: &mut Pipeline) {
        panic!("Cannot unload the dummy state!");
    }
    
    fn update(&mut self) -> UpdateResult {
        panic!("Cannot update the dummy state!");
    }
    
    fn render(&self,_graphics: &Graphics,_pipeline: &mut Pipeline) {
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
pub struct MousePoint {
    x: i32,
    y: i32
}

fn placeholder_state_generator(_graphics: &Graphics,_pipeline: &mut Pipeline) -> AppState {
    panic!("Cannot generate an AppState using the placeholder state generator");
}

pub struct AppCreationOptions {
    pub state_generator: AppStateGenerator,
    pub pipeline_options: Option<PipelineCreationOptions>,
    pub log_trace_config: Option<LogTraceConfig>,
}

pub fn create_app(options: AppCreationOptions) -> App {
    return App {
        handles: None,

        state_generator: options.state_generator,
        state: Box::new(DummyState),

        state_loaded: false,
        surface_configured: false,

        app_exiting: false,

        pipeline_options: options.pipeline_options,

        window_width: 0,
        window_height: 0,

        frame_number: 0,
        event_number: 0,
        
        mouse_point: MousePoint { x: 0, y: 0 },
 
        /* For Debugging */
        log_trace_config: match options.log_trace_config {
            Some(value) => value,
            None => Default::default(),
        }
    }
}

fn get_center_position(parent: PhysicalSize<u32>,child: PhysicalSize<u32>) -> Position {
    let x = (parent.width - child.width) / 2;
    let y = (parent.height - child.height) / 2;
    return Position::Physical(PhysicalPosition::new(x as i32,y as i32));
}

impl App {
    fn try_configure_surface(&mut self) -> bool {
        let width = self.window_width;
        let height = self.window_height;

        if width < 1 || height < 1 {
            log::warn!("Cannot configure surface with one or more of the following size components: {}x{}",width,height);
            return false;
        }

        if let Some(handles) = &mut self.handles {
            handles.graphics.configure_surface_size(width,height);
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
        return match self.state.update() {
            UpdateResult {
                operation: AppOperation::Continue,
                new_state: None
            } => EventLoopOperation::Continue,

            UpdateResult {
                operation: AppOperation::Transition,
                new_state: Some(state_generator)
            } => {
                self.unload_state();
                self.state_generator = state_generator;
                return EventLoopOperation::Repeat;
            }

            UpdateResult {
                operation: AppOperation::Terminate,
                new_state: None
            } => EventLoopOperation::Terminate,

            /* These invalid results can probably be fixed up by a better creation pattern. I.e. don't let the caller write them manually. */

            UpdateResult {
                new_state: Some(_),
                ..
            } => {
                log::error!("Invalid update result: A state has been provided, but it has not been provided with a transition instruction. Triggering app termination.");
                return EventLoopOperation::Terminate;
            },

            UpdateResult {
                operation: AppOperation::Transition,
                new_state: None
            } => {
                log::error!("Invalid update result: A transition has been requested, but a state has not been provided. Triggering app termination.");
                return EventLoopOperation::Terminate;
            },
        };
    }

    /* Primary function to handle updating */
    fn handle_redraw(&mut self,event_loop: &ActiveEventLoop) {
        if self.log_trace_config.redraw {
            log::trace!("handle_redraw - frame_number:{} | event_number:{}",self.frame_number,self.event_number);
        }

        /* Prevent a stack overflow in the case of extremely cursed state loading. */
        loop {
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
                self.state.render(&handles.graphics,&mut handles.pipeline);
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
            let new_state = (self.state_generator)(&handles.graphics,&mut handles.pipeline);
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
            self.state.unload(&handles.graphics,&mut handles.pipeline);
            self.handles = Some(handles);    
        } else if !self.app_exiting {
            log::warn!("Unusual app exit state. The app handles have already been dropped.");
            return;
        }
        self.state = Box::new(DummyState);
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

    fn handle_mouse_move(&mut self,point: MousePoint) {
        if self.log_trace_config.mouse_move {
            log::trace!("handle_mouse_move - frame_number:{} | event_number:{} | x:{} y:{}",self.frame_number,self.event_number,point.x,point.y);
        }
        self.mouse_point = point;
        if !self.state_loaded {
            return;
        }
        //TODO
    }
}

impl ApplicationHandler for App {

    /* Create window and graphics pipeline. */
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

        let graphics = match pollster::block_on(Graphics::new(window.clone())) {
            Ok(graphics) => graphics,
            Err(error) => {
                log::error!("{}",error);
                self.terminate_app(event_loop);
                return;
            }
        };
        window.set_visible(true);

        let pipeline = Pipeline::create(&graphics,match self.pipeline_options.take() { /* We take so the underlying vectors (if any) are dropped. */
            Some(options) => options,
            None => PipelineCreationOptions::default(),
        });

        self.handles = Some(PostConstructionHandles { window, graphics, pipeline });

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
            DeviceEvent::MouseMotion { delta } => self.state.input(InputEvent::MouseMoveRaw(delta)),
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
                self.handle_mouse_move(MousePoint {x: position.x as i32,y: position.y as i32});
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

            WindowEvent::CloseRequested | WindowEvent::Destroyed => self.terminate_app(event_loop), //Goodbye, cruel world.

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

            /* I actually have no idea what the fuck this event means. */
            WindowEvent::ActivationTokenDone {
                token,
                serial: _,
            } => {
                if self.log_trace_config.other {
                    let token_string = token.into_raw();
                    log::trace!("activation token done - frame_number:{} | event_number:{} | token:{}",self.frame_number,self.event_number,token_string);
                }
            },

            /* TODO: Might want to use this ? */
            WindowEvent::ScaleFactorChanged { .. } => {
                if self.log_trace_config.other {
                    log::trace!("scale factor changed - frame_number:{} | event_number:{}",self.frame_number,self.event_number);
                }
            }

            WindowEvent::ThemeChanged(_) => {
                if self.log_trace_config.other {
                    log::trace!("theme changed - frame_number:{} | event_number:{}",self.frame_number,self.event_number);
                }
            },

            WindowEvent::Occluded(occluded) => {
                if self.log_trace_config.other {
                    log::trace!("occluded - frame_number:{} | event_number:{} | occlusion:{}",self.frame_number,self.event_number,occluded);
                }
            },

            _ => {}
        };

        self.event_number += 1;
    }
}
