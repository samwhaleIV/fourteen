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
    context: GraphicsContext
}

pub type SharedStateGenerator<TSharedState> = fn(&VirtualDevice,&mut GraphicsContext) -> TSharedState;

pub struct App<TSharedState> {

    handles: Option<LongLifetimeHandles>,
    state: Option<AppState<TSharedState>>,
    shared_state: Option<TSharedState>,

    state_generator: AppStateGenerator<TSharedState>,
    shared_state_generator: SharedStateGenerator<TSharedState>,

    surface_configured: bool,
    app_exiting: bool,

    frame_number: u128,
    event_number: u128,

    window_width: u32,
    window_height: u32,

    mouse_point: (f32,f32),

    log_trace_config: LogTraceConfig,
    context_options: Option<GraphicsContextConfiguration>,
}

enum EventLoopOperation {
    Continue,
    Terminate,
    Repeat
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

fn placeholder_state_generator<TSharedState>(_device: &VirtualDevice,_pipeline: &mut GraphicsContext) -> AppState<TSharedState> {
    panic!("Cannot generate an AppState using the placeholder state generator");
}

pub struct AppConfiguration<TSharedState> {
    pub state_generator: AppStateGenerator<TSharedState>,
    pub shared_state_generator: SharedStateGenerator<TSharedState>,
    pub context_options: Option<GraphicsContextConfiguration>,
    pub log_trace_config: Option<LogTraceConfig>,
}

fn get_center_position(parent: PhysicalSize<u32>,child: PhysicalSize<u32>) -> Position {
    let x = (parent.width - child.width) / 2;
    let y = (parent.height - child.height) / 2;
    return Position::Physical(PhysicalPosition::new(x as i32,y as i32));
}

struct BorrowingBlock<TSharedState> {
    pub state: AppState<TSharedState>,
    pub shared_state: Option<TSharedState>,
    pub handles: LongLifetimeHandles,
}

impl<TSharedState> BorrowingBlock<TSharedState> {

    fn insert_shared_state(&mut self) {
        self.state.insert_shared_state(self.shared_state.take());
    }

    fn remove_shared_state(&mut self) {
        self.shared_state = self.state.remove_shared_state();
    }

    pub fn update_state(&mut self) -> UpdateResult<TSharedState> {
        self.insert_shared_state();
        let result = self.state.update(&self.handles.device,&mut self.handles.context);
        self.remove_shared_state();
        return result;
    }

    pub fn render_state(&mut self) {
        self.insert_shared_state();
        self.state.render(&self.handles.device,&mut self.handles.context);
        self.remove_shared_state();
    }

    pub fn unload_state(&mut self) {
        self.insert_shared_state();
        self.state.unload(&self.handles.device,&mut self.handles.context);
        self.remove_shared_state();
    }
}

impl<TSharedState> App<TSharedState> {

    pub fn create(options: AppConfiguration<TSharedState>) -> Self {
        return Self {
            handles: None,
            state: None,
            shared_state: None,

            shared_state_generator: options.shared_state_generator,
            state_generator: options.state_generator,

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

    fn try_get_borrowing_block(&mut self) -> Option<BorrowingBlock<TSharedState>> {
        let handles = self.handles.take();
        let state = self.state.take();
        let shared_state = self.shared_state.take();

        if handles.is_none() || state.is_none() || shared_state.is_none() {
            if handles.is_none() {
                log::error!("App handles not found!");
            }
            if state.is_none() {
                log::error!("App state is missing!");
            }
            if shared_state.is_none() {
                log::error!("Shared state is missing!");
            }
            self.handles = handles;
            self.state = state;
            self.shared_state = shared_state;
            return None;
        }
        if let (Some(handles),Some(state)) = (handles,state) {
            return Some(BorrowingBlock {
                state,
                handles,
                shared_state,
            });
        } else {
            return None;
        }
    }

    fn return_borrowing_block(&mut self,borrowing_block: BorrowingBlock<TSharedState>) {
        self.handles = Some(borrowing_block.handles);
        self.state = Some(borrowing_block.state);
        self.shared_state = borrowing_block.shared_state;
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
        /* Load a state if ones does not exist. */
        if self.state.is_none() {
            if let Some(mut handles) = self.handles.take() {
                let new_state = (self.state_generator)(&handles.device,&mut handles.context);
                self.state_generator = placeholder_state_generator;
                self.state = Some(new_state);
                self.handles = Some(handles);
            } else {
                log::error!("App handles not found!");
                return EventLoopOperation::Terminate;
            }
        }
        if let Some(mut borrowing_block) = self.try_get_borrowing_block() {

            let update_result = borrowing_block.update_state();
            self.return_borrowing_block(borrowing_block);

            let event_loop_operation = match update_result.get_operation() {
                AppStateOperation::Continue => {
                    EventLoopOperation::Continue
                },

                AppStateOperation::Terminate => {
                    EventLoopOperation::Terminate
                },

                AppStateOperation::Transition => {
                    if let Some(state_generator) = update_result.get_state_generator() {
                        self.unload_state();
                        self.state_generator = state_generator;
                        EventLoopOperation::Repeat
                    } else {
                        log::error!("Missing app state transition data!");
                        EventLoopOperation::Terminate
                    }
                }
            };
            return event_loop_operation;
        } else {
            return EventLoopOperation::Terminate;
        }
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

        if let Some(mut borrowing_block) = self.try_get_borrowing_block() {
            if self.surface_configured || self.try_configure_surface() {
                borrowing_block.render_state();
            }
            borrowing_block.handles.window.request_redraw();
            self.return_borrowing_block(borrowing_block);
        } else {
            self.terminate_app(event_loop);
            return;
        }

        self.frame_number += 1;
    }

    fn unload_state(&mut self) {
        if let Some(mut borrowing_block) = self.try_get_borrowing_block() {
            borrowing_block.unload_state();
            self.return_borrowing_block(borrowing_block);
            self.state = None;
        }
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
        log::info!("Termination success; event loop exiting.");
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
        if !self.state.is_none() {
            return;
        }
        //TODO
    }

    fn handle_mouse_release(&mut self) {
        if self.log_trace_config.mouse_click {
            log::trace!("handle_mouse_release - frame_number:{} | event_number:{}",self.frame_number,self.event_number);
        }
        if !self.state.is_none() {
            return;
        }
        //TODO
    }

    fn handle_mouse_move(&mut self,point: (f32,f32)) {
        if self.log_trace_config.mouse_move {
            log::trace!("handle_mouse_move - frame_number:{} | event_number:{} | x:{} y:{}",self.frame_number,self.event_number,point.0,point.1);
        }
        self.mouse_point = point;
        if !self.state.is_none() {
            return;
        }
        //TODO
    }
}

impl<TSharedState> ApplicationHandler for App<TSharedState> {

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {

        if self.handles.is_some() {
            /* This shouldn't happen on desktop platforms. */
            log::info!("The app has been resumed. Welcome back!");
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
                log::error!("Scary error: {}",error);
                self.terminate_app(event_loop);
                return;
            }
        };
        window.set_visible(true);

        let mut context = GraphicsContext::create(&device,match self.context_options.take() { /* We take so the underlying vectors (if any) are dropped. */
            Some(options) => options,
            None => GraphicsContextConfiguration::default(),
        });

        self.shared_state = Some((self.shared_state_generator)(&device,&mut context));

        self.handles = Some(LongLifetimeHandles {
            window,
            device,
            context
        });

        log::info!("Long living app handles are now configured.");
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        /* This shouldn't happen on desktop platforms. */
        log::info!("App suspended. Goodnight, sweet prince.");
    }

    fn device_event(&mut self,_event_loop: &ActiveEventLoop,_device_id: DeviceId,event: DeviceEvent) {
        if self.app_exiting || self.state.is_none() {
            return;
        }
        if let Some(mut state) = self.state.take() {
            match event {
                DeviceEvent::MouseMotion { delta } => state.input(InputEvent::MouseMoveDelta((delta.0 as f32,delta.1 as f32))),
                _ => {}
            }
            self.state = Some(state);
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
