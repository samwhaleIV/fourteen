
const WINDOW_TITLE: &'static str = include_str!("../config/window_title.txt");
const MINIMUM_WINDOW_SIZE: (u32,u32) = (400,300);

use std::{sync::Arc, time::{Instant}};

use crate::{point::Point};
use crate::graphics::{Graphics,PipelineVariant};

use wgpu::{CommandEncoder, RenderPass, TextureView};

use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize, Position},
    event::*,
    event_loop::{ActiveEventLoop},
    keyboard::{KeyCode,PhysicalKey},
    window::{Window,WindowId}
};

pub struct UpdateResult {
    pub operation: AppOperation,
    pub new_state: Option<AppStateGenerator>
}

impl Default for UpdateResult {
    fn default() -> Self {
        return UpdateResult {
            operation: AppOperation::Continue,
            new_state: None 
        }
    }
}

pub trait AppStateHandler {
    fn unload(&mut self,graphics: &Graphics);
    fn update(&mut self) -> UpdateResult;
    fn render(&mut self,render_pass: &mut RenderPass);
}

pub enum AppOperation {
    Continue,
    Terminate,
    Transition
}

enum EventLoopOperation {
    Continue,
    Terminate,
    Repeat
}

pub enum RenderPassMode {
    Basic
}

pub type AppState = Box<dyn AppStateHandler>;
pub type AppStateGenerator = fn(&Graphics) -> AppState;

pub struct App {
    state_loaded: bool,
    surface_configured: bool,
    app_exiting: bool,

    frame_number: u128,
    event_number: u128,

    window_width: u32,
    window_height: u32,

    window: Option<Arc<Window>>,
    graphics: Option<Graphics>,

    start_time: Instant,
    render_pass_mode: RenderPassMode,
 
    state_generator: AppStateGenerator,
    state: AppState,
    
    log_trace_config: LogTraceConfig
}

struct DummyState;

impl AppStateHandler for DummyState {
    /* Oh no! You've activated my trap card. */
    fn unload(&mut self,_graphics: &Graphics) {
        panic!("Cannot unload the dummy state!");
    }
    fn update(&mut self) -> UpdateResult {
        panic!("Cannot update the dummy state!");
    }
    fn render(&mut self,_render_pass: &mut RenderPass) {
        panic!("Cannot render the dummy state!");
    }
}

fn placeholder_state_generator(_: &Graphics) -> AppState {
    panic!("Cannot generate an AppState using the placeholder state generator");
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

pub fn create_app(state_generator: AppStateGenerator,log_trace_config: LogTraceConfig) -> App {
    return App {
        window: None,
        graphics: None,

        start_time: Instant::now(),
        render_pass_mode: RenderPassMode::Basic,

        state_generator,
        state: Box::new(DummyState),

        state_loaded: false,
        surface_configured: false,

        app_exiting: false,

        window_width: 0,
        window_height: 0,

        frame_number: 0,
        event_number: 0,
 
        /* For Debugging */
        log_trace_config
    }
}

fn get_basic_render_pass<'a>(encoder: &'a mut CommandEncoder,view: &'a TextureView) -> RenderPass<'a> {
    return encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        occlusion_query_set: None,
        timestamp_writes: None,
    });
}

fn get_center_position(parent: PhysicalSize<u32>,child: PhysicalSize<u32>) -> Position {
    let x = (parent.width - child.width) / 2;
    let y = (parent.height - child.height) / 2;
    return Position::Physical(PhysicalPosition::new(x as i32,y as i32));
}

impl App {
    fn configure_surface(&mut self) {
        let width = self.window_width;
        let height = self.window_height;

        if width < 1 || height < 1 {
            log::warn!("Cannot configure surface with one or more of the following size components: {}x{}",width,height);
            return;
        }

        let graphics = self.graphics.as_mut().unwrap();
        
        graphics.config.width = width;
        graphics.config.height = height;

        graphics.surface.configure(&graphics.device,&graphics.config);

        self.surface_configured = true;
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

            _ => {
                log::error!("Invalid update result: This operation has not been implemented. (This is not the caller's fault.) Triggering app termination.");
                return EventLoopOperation::Terminate;
            }
        };
    }

    fn load_state(&mut self) {
        if self.state_loaded {
            log::warn!("Cannot load state, we are already in a loaded state.");
            return;
        }
        let new_state = (self.state_generator)(self.graphics.as_ref().unwrap());
        self.state_generator = placeholder_state_generator;
        self.state = new_state;
        self.state_loaded = true;
    }

    fn unload_state(&mut self) {
        if !self.state_loaded {
            if !self.app_exiting {
                log::warn!("Cannot unload state, we are already in an unloaded state.");
            }
            return;
        }
        self.state.unload(self.graphics.as_ref().unwrap());
        self.state = Box::new(DummyState);
        self.state_loaded = false;
    }

    fn render(&mut self) -> Result<(),wgpu::SurfaceError> {
        let graphics = self.graphics.as_mut().unwrap();

        let output = graphics.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = graphics.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = match self.render_pass_mode {
                RenderPassMode::Basic => get_basic_render_pass(&mut encoder,&view),
                _ => get_basic_render_pass(&mut encoder,&view)
            };

            render_pass.set_pipeline(&graphics.render_pipeline);
            self.state.render(&mut render_pass);
        }

        graphics.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn terminate_app(&mut self,event_loop: &ActiveEventLoop) {
        if self.app_exiting {
            log::warn!("App termination is already marked.");
            return;
        }
        log::info!("Terminating app; it's the right thing to do. All programs must die eventually.");
        self.app_exiting = true;
        self.unload_state();
        event_loop.exit();
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

        if self.state_loaded {
            if !self.surface_configured {
                self.configure_surface();
            }

            if self.surface_configured {
                match self.render() {
                    Ok(_) => {},
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        log::warn!("WebGPU surface error. Is the surface lost or outdated? Attempting to configure surface again.");
                        self.configure_surface()
                    },
                    Err(error) => log::error!("Unable to render: {}",error)
                }
            }
        } else {
            log::warn!("Attempt to render without a loaded state.");
        }

        self.window.as_mut().unwrap().request_redraw();
        self.frame_number += 1;
    }

    fn handle_key_change(&mut self,_code: KeyCode,_pressed: bool){
        if self.log_trace_config.key_change {
            log::trace!("handle_key_change - frame_number:{} | event_number:{}",self.frame_number,self.event_number);
        }
    }

    fn handle_mouse_press(&mut self) {
        if self.log_trace_config.mouse_click {
            log::trace!("handle_mouse_press - frame_number:{} | event_number:{}",self.frame_number,self.event_number);
        }
    }

    fn handle_mouse_release(&mut self) {
        if self.log_trace_config.mouse_click {
            log::trace!("handle_mouse_release - frame_number:{} | event_number:{}",self.frame_number,self.event_number);
        }
    }

    fn handle_mouse_move(&mut self,_point: Point) {
        if self.log_trace_config.mouse_move {
            log::trace!("handle_mouse_move - frame_number:{} | event_number:{}",self.frame_number,self.event_number);
        }
    }

    fn handle_mouse_enter(&mut self) {
        if self.log_trace_config.mouse_over_window {
            log::trace!("handle_mouse_enter - frame_number:{} | event_number:{}",self.frame_number,self.event_number);
        }
    }

    fn handle_mouse_leave(&mut self) {
        if self.log_trace_config.mouse_over_window {
            log::trace!("handle_mouse_leave - frame_number:{} | event_number:{}",self.frame_number,self.event_number);
        }
    }
}

impl ApplicationHandler for App {

    /* Create window and graphics pipeline. */
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {

        if self.window.is_some() {
            /* This shouldn't happen on desktop platforms. */
            log::info!("The app has been resumed. Welcome back.");
            return;
        } else {
            log::info!("Received 'resumed' call. Getting on with the window and graphics setup.");
        }
        
        let (min_width,min_height) = MINIMUM_WINDOW_SIZE;
        let min_inner_size = PhysicalSize::new(min_width,min_height);
        let window_size = PhysicalSize::new(min_width * 2,min_height * 2); /* TODO: Load last window size of application. */

        let window_attributes = Window::default_attributes()
            .with_title(WINDOW_TITLE)
            .with_min_inner_size(min_inner_size)
            .with_inner_size(window_size)
            .with_visible(false);

        let window = match event_loop.create_window(window_attributes) {
            Ok(window) => window,
            Err(error) => {
                log::error!("Could not create window through event loop: {}",error);
                self.terminate_app(event_loop);
                return;
            }
        };

        /* It's okay that this is optional; it's only used to center the window. */
        if let Some(monitor) = window.current_monitor() {
            let position = get_center_position(monitor.size(),window.outer_size());
            window.set_outer_position(position);
        }

        let arc_window = Arc::new(window);
        let pipeline_variant = PipelineVariant::Basic;     
        let graphics = pollster::block_on(Graphics::new(arc_window.clone(),pipeline_variant)).unwrap();
        arc_window.set_visible(true);

        self.window = Some(arc_window);
        self.graphics = Some(graphics);

        log::info!("Graphics pipeline and window are now configured.");
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        /* This shouldn't happen on desktop platforms. */
        log::info!("App suspended. Goodnight, sweet prince.");
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

                self.configure_surface();
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

            WindowEvent::CursorMoved {
                position,
                device_id: _
            } => self.handle_mouse_move(Point {
                x: position.x as i32,
                y: position.y as i32
            }),

            WindowEvent::CursorEntered { device_id: _ } => self.handle_mouse_enter(),
            WindowEvent::CursorLeft { device_id: _ } => self.handle_mouse_leave(),

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
