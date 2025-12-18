const WINDOW_TITLE: &'static str = "Twelve Engine - Hello, World!";
const MINIMUM_WINDOW_SIZE: (u32,u32) = (800,600);
const MAX_STATE_LOAD_PASSES: u32 = 32;

use std::sync::Arc;

use super::app_state::*;

use crate::{
    app::VirtualDevice,
    wgpu::{
        GraphicsContext,
        GraphicsContextConfiguration,
        GraphicsContextInternal,
    }
};

use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize, Position},
    event::*,
    event_loop::{ActiveEventLoop},
    keyboard::{PhysicalKey},
    window::{Window,WindowId}
};

pub type AppState<TSharedState> = Box<dyn AppStateInterface<TSharedState>>;
pub type AppStateGenerator<TSharedState> = fn(&mut AppContext<TSharedState>) -> AppState<TSharedState>;
pub type SharedStateGenerator<TSharedState> = fn(&mut GraphicsContext<VirtualDevice>) -> TSharedState;

struct TransientBlock<TSharedState> {
    window: Option<Arc<Window>>,
    device: Option<VirtualDevice>,
    graphics_context: Option<GraphicsContext<VirtualDevice>>,
    app_state: Option<AppState<TSharedState>>,
    shared_state: Option<TSharedState>,
}

impl<TSharedState> Default for TransientBlock<TSharedState> {
    fn default() -> Self {
        return Self {
            window: None,
            device: None,
            graphics_context: None,
            app_state: None,
            shared_state: None
        }
    }
}
pub struct App<TSharedState> {

    transient_block: TransientBlock<TSharedState>,

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

    received_resume_call: bool
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

fn placeholder_state_generator<TSharedState>(_context: &mut AppContext<TSharedState>) -> AppState<TSharedState> {
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
    window: Arc<Window>,
    app_state: AppState<TSharedState>,
    device: Option<VirtualDevice>,
    graphics_context: Option<GraphicsContext<VirtualDevice>>,
    shared_state: Option<TSharedState>,
}

impl<TSharedState> BorrowingBlock<TSharedState> {
    pub fn try_get_app_context(&mut self) -> Option<AppContext<TSharedState>> {
        if
            let (Some(shared_state),Some(mut graphics_context),Some(device)) = (self.shared_state.take(),self.graphics_context.take(),self.device.take())
        {
            graphics_context.insert_wgpu_handle(device);
            return Some(AppContext::construct(shared_state,graphics_context));
        } else {
            log::warn!("Borrowing block is missing parts of the app context.");
            return None;
        }
    }
    pub fn return_app_context(&mut self,app_context: AppContext<TSharedState>) {
        let (shared_state,mut graphics_context) = app_context.deconstruct();
        self.shared_state = Some(shared_state);
        self.device = graphics_context.remove_wgpu_handle();
        self.graphics_context = Some(graphics_context);
    }
}

impl<TSharedState> App<TSharedState> {
    fn try_get_borrowing_block(&mut self) -> Option<BorrowingBlock<TSharedState>> {
        let TransientBlock {
            window: Some(window),
            device: Some(device),
            graphics_context: Some(graphics_context),
            app_state: Some(app_state),
            shared_state: Some(shared_state),
        } = std::mem::take(&mut self.transient_block) else {
            return None;
        };
        return Some(BorrowingBlock {
            window,
            app_state,
            device: Some(device),
            graphics_context: Some(graphics_context),
            shared_state: Some(shared_state),
        });
    }

    fn return_borrowing_block(&mut self,borrowing_block: BorrowingBlock<TSharedState>) {
        self.transient_block = TransientBlock {
            window: Some(borrowing_block.window),
            app_state: Some(borrowing_block.app_state),
            device: borrowing_block.device,
            graphics_context: borrowing_block.graphics_context,
            shared_state: borrowing_block.shared_state
        };
    }

    pub fn create(options: AppConfiguration<TSharedState>) -> Self {
        return Self {
            transient_block: TransientBlock::<TSharedState>::default(),

            shared_state_generator: options.shared_state_generator,
            state_generator: options.state_generator,

            surface_configured: false,
            received_resume_call: false,

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

        if let Some(mut device) = self.transient_block.device.take() {
            device.configure_surface_size(width,height);
            self.transient_block.device = Some(device);
        } else {
            log::error!("Virtual device not found.");
            return false;
        }

        self.surface_configured = true;
        return true;
    }

    fn try_load_new_state(&mut self) -> bool {
        if 
            let Some(mut borrowing_block) = self.try_get_borrowing_block() &&
            let Some(mut app_context) = borrowing_block.try_get_app_context() 
        {
            let new_state = (self.state_generator)(&mut app_context);
            self.state_generator = placeholder_state_generator;

            borrowing_block.return_app_context(app_context);
            self.return_borrowing_block(borrowing_block);

            self.transient_block.app_state = Some(new_state);
        } else {
            return false;
        }
        return true;
    }

    fn update(&mut self) -> EventLoopOperation {
        /* Load a state if ones does not exist. */
        if self.transient_block.app_state.is_none() && !self.try_load_new_state() {
            return EventLoopOperation::Terminate;
        }
        if 
            let Some(mut borrowing_block) = self.try_get_borrowing_block() &&
            let Some(mut app_context) = borrowing_block.try_get_app_context() 
        {
            let update_result = borrowing_block.app_state.update(&mut app_context);
            borrowing_block.return_app_context(app_context);
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
        if
            let Some(mut borrowing_block) = self.try_get_borrowing_block() &&
            let Some(mut app_context) = borrowing_block.try_get_app_context() 
        {
            if self.surface_configured || self.try_configure_surface() {
                borrowing_block.app_state.render(&mut app_context);
            }
            borrowing_block.window.request_redraw();
            borrowing_block.return_app_context(app_context);
            self.return_borrowing_block(borrowing_block);
        } else {
            self.terminate_app(event_loop);
            return;
        }
        self.frame_number += 1;
    }

    fn unload_state(&mut self) {
        if 
            let Some(mut borrowing_block) = self.try_get_borrowing_block() &&
            let Some(mut app_context) = borrowing_block.try_get_app_context() 
        {
            borrowing_block.app_state.unload(&mut app_context);
            borrowing_block.return_app_context(app_context);
            self.return_borrowing_block(borrowing_block);
            self.transient_block.app_state = None;
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

    fn send_input(&mut self,input_event: InputEvent) {
        if
            let Some(mut borrowing_block) = self.try_get_borrowing_block() &&
            let Some(mut app_context) = borrowing_block.try_get_app_context() 
        {    
            borrowing_block.app_state.input(input_event,&mut app_context);
            borrowing_block.return_app_context(app_context);
            self.return_borrowing_block(borrowing_block);
        } else {
            log::warn!("Could not send input to app state because the context is missing.");
        }
    }
}

impl<TSharedState> ApplicationHandler for App<TSharedState> {

    fn resumed(&mut self,event_loop: &ActiveEventLoop) {

        if self.received_resume_call {
            /* This shouldn't happen on desktop platforms. */
            log::info!("The app has been resumed. Welcome back!");
            return;
        } else {
            log::info!("Received 'resumed' call. Getting on with the window and graphics setup.");
        }
        self.received_resume_call = true;
        
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

        let mut graphics_context = GraphicsContext::create(&device,match self.context_options.take() { /* We take so the underlying vectors (if any) are dropped. */
            Some(options) => options,
            None => GraphicsContextConfiguration::default(),
        });

        let shared_state = (self.shared_state_generator)(&mut graphics_context);

        self.transient_block.window = Some(window);
        self.transient_block.device = Some(device);

        self.transient_block.graphics_context = Some(graphics_context);
        self.transient_block.shared_state = Some(shared_state);

        log::info!("App graphics context and shared state are configured.");
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        /* This shouldn't happen on desktop platforms. */
        log::info!("App suspended. Goodnight, sweet prince.");
    }

    fn device_event(&mut self,_event_loop: &ActiveEventLoop,_device_id: DeviceId,event: DeviceEvent) {
        if self.app_exiting || self.transient_block.app_state.is_none() {
            return;
        }
        match event {
            DeviceEvent::MouseMotion { delta } => {
                self.send_input(InputEvent::MouseMoveDelta((delta.0 as f32,delta.1 as f32)));
            },
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
                    state: ElementState::Pressed,
                    repeat: false,
                    ..
                },
                device_id: _
            } => self.send_input(InputEvent::KeyPress(code)),

            WindowEvent::KeyboardInput {
                is_synthetic: false,
                event: KeyEvent {
                    physical_key: PhysicalKey::Code(code),
                    state: ElementState::Released,
                    repeat: false,
                    ..
                },
                device_id: _
            } => self.send_input(InputEvent::KeyRelease(code)),

            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state: ElementState::Pressed,
                device_id: _
            } => self.send_input(InputEvent::MousePress(self.mouse_point)),

            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state: ElementState::Released,
                device_id: _
            } => self.send_input(InputEvent::MousePress(self.mouse_point)),

            WindowEvent::CursorMoved { position, device_id: _ } => {
                self.mouse_point = (position.x as f32,position.y as f32);
                self.send_input(InputEvent::MouseMove(self.mouse_point));
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
