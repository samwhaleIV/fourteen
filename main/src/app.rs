
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
    pub new_state: Option<AppState>
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
    fn load(&mut self,graphics: &Graphics);
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

enum StateTransitionPhase {
    FirstStateUnloaded
}

pub enum RenderPassMode {
    Basic
}

type AppState = Box<dyn AppStateHandler>;

pub struct App {
    window: Option<Arc<Window>>,
    graphics: Option<Graphics>,

    start_time: Instant,
    render_pass_mode: RenderPassMode,

    state: AppState,

    state_loaded: bool,
    surface_configured: bool,

    frame_number: u128,
    event_number: u128,

    app_exiting: bool,

    width: u32,
    height: u32,

    log_trace_redraw: bool,
    log_trace_mouse_move: bool,
    log_trace_window_move: bool
}

pub fn create_app(start_state: Box<dyn AppStateHandler>) -> App {
    return App {
        window: None,
        graphics: None,

        start_time: Instant::now(),
        render_pass_mode: RenderPassMode::Basic,

        state: start_state,

        state_loaded: false,
        surface_configured: false,

        app_exiting: false,

        width: 0,
        height: 0,

        frame_number: 0,
        event_number: 0,

        /* For Debugging */
        log_trace_redraw: false,
        log_trace_mouse_move: false,
        log_trace_window_move: false
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
        let width = self.width;
        let height = self.height;

        if width < 1 || height < 1 {
            return;
        }
        
        let graphics = match &mut self.graphics {
            Some(graphics) => graphics,
            None => panic!("Graphics should already exist before a resize operation.")
        };

        graphics.config.width = width;
        graphics.config.height = height;

        graphics.surface.configure(&graphics.device,&graphics.config);

        self.surface_configured = true;
    }

    fn update(&mut self) -> EventLoopOperation {
        let graphics = match &self.graphics {
            Some(graphics) => graphics,
            _ => return EventLoopOperation::Continue
        };
        if !self.state_loaded {
            self.state.load(graphics);
            self.state_loaded = true;
        }
        return match self.state.update() {
            UpdateResult {
                operation: AppOperation::Continue,
                new_state: None
            } => EventLoopOperation::Continue,

            UpdateResult {
                operation: AppOperation::Transition,
                new_state: Some(state)
            } => {
                self.state.unload(&graphics);
                self.state_loaded = false;
                self.state = state;
                return EventLoopOperation::Repeat;
            }

            UpdateResult {
                operation: AppOperation::Terminate,
                new_state: None
            } => EventLoopOperation::Terminate,

            UpdateResult {
                new_state: Some(_),
                ..
            } => panic!("Invalid app operation. A state has been provided, but it has not been provided with a transition instruction."),

            UpdateResult {
                operation: AppOperation::Transition,
                new_state: None
            } => panic!("Missing a state for the requested transition. What are you trying to do? Make it make sense."),

            _ => panic!("Unimplemented app operation. This is not the caller's fault. Don't panic. We can fix this.")
        };
    }

    fn unload(&mut self) {
        let graphics = match &self.graphics {
            Some(graphics) => graphics,
            None => panic!("Missing graphics state when needed for state unloader.")
        };

        if !self.state_loaded {
            panic!("State is already unloaded when it shouldn't be.");
        }

        self.state.unload(graphics);
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
 
    /* Primary function to handle updating */
    fn handle_redraw(&mut self,event_loop: &ActiveEventLoop) {
        if self.log_trace_redraw {
            log::trace!("handle_redraw - frame_number:{} | event_number:{}",self.frame_number,self.event_number);
        }

        /* Prevent a stack overflow in the case of extremely cursed state loading. */
        loop {
            match self.update() {
                EventLoopOperation::Continue => break,
                EventLoopOperation::Terminate => {
                    self.unload();
                    event_loop.exit();
                    return;
                },
                EventLoopOperation::Repeat => continue
            }
        }

        if !self.state_loaded {
            panic!("State is not loaded by a point in which it should definitely be loaded.");
        }
        
        if !self.surface_configured {
            self.configure_surface();
        }

        if self.surface_configured {
            match self.render() {
                Ok(_) => {},
                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => self.configure_surface(),
                Err(error) => log::error!("Unable to render: {}",error)
            }
        }

        if let Some(window) = &mut self.window {
            window.request_redraw();
        }
        
        self.frame_number += 1;
    }

    fn handle_key_change(&mut self,_code: KeyCode,_pressed: bool){
        log::trace!("handle_key_change - frame_number:{} | event_number:{}",self.frame_number,self.event_number);
    }

    fn handle_mouse_press(&mut self) {
        log::trace!("handle_mouse_press - frame_number:{} | event_number:{}",self.frame_number,self.event_number);
    }

    fn handle_mouse_release(&mut self) {
        log::trace!("handle_mouse_release - frame_number:{} | event_number:{}",self.frame_number,self.event_number);
    }

    fn handle_mouse_move(&mut self,_point: Point) {
        if self.log_trace_mouse_move {
            log::trace!("handle_mouse_move - frame_number:{} | event_number:{}",self.frame_number,self.event_number);
        }
    }

    fn handle_mouse_enter(&mut self) {
        log::trace!("handle_mouse_enter - frame_number:{} | event_number:{}",self.frame_number,self.event_number);
    }

    fn handle_mouse_leave(&mut self) {
        log::trace!("handle_mouse_leave - frame_number:{} | event_number:{}",self.frame_number,self.event_number);
    }
}

impl ApplicationHandler for App {

    /* Create window and graphics pipeline. */
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {

        if self.window.is_some() {
            log::info!("The window has been resumed. Welcome back.");
            return;
        }
        
        let (min_width,min_height) = MINIMUM_WINDOW_SIZE;
        let min_inner_size = PhysicalSize::new(min_width,min_height);
        let window_size = PhysicalSize::new(min_width * 2,min_height * 2); /* TODO: Load last window size of application. */

        let window_attributes = Window::default_attributes()
            .with_title(WINDOW_TITLE)
            .with_min_inner_size(min_inner_size)
            .with_inner_size(window_size)
            .with_visible(false);

        let window = event_loop.create_window(window_attributes).unwrap();

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
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        log::info!("Window suspended. Goodnight, sweet prince.");
    }

    fn window_event(&mut self,event_loop: &ActiveEventLoop,_window_id: WindowId,event: WindowEvent) {
        if self.app_exiting {
            return;
        }
        match event {
            WindowEvent::RedrawRequested => self.handle_redraw(event_loop),

            WindowEvent::Resized(size) => {          
                log::trace!("resized - frame_number:{} | event_number:{} | {}x{}",self.frame_number,self.event_number,size.width,size.height);

                self.width = size.width;
                self.height = size.height;

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

            WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                self.app_exiting = true;
                self.unload();
                event_loop.exit();
            },

            WindowEvent::Focused(focused) => match focused {
                true => log::trace!("window focused - frame_number:{} | event_number:{}",self.frame_number,self.event_number),

                false => log::trace!("window lost focus - frame_number:{} | event_number:{}",self.frame_number,self.event_number),
            },

            WindowEvent::ActivationTokenDone {
                token,
                serial: _,
            } => {
                let token_string = token.into_raw();
                log::trace!("activation token done - frame_number:{} | event_number:{} | token:{}",self.frame_number,self.event_number,token_string);
            },

            WindowEvent::ScaleFactorChanged {
                scale_factor: _,
                inner_size_writer: _
            } => log::trace!("scale factor changed - frame_number:{} | event_number:{}",self.frame_number,self.event_number),

            WindowEvent::ThemeChanged(_) => log::trace!("theme changed - frame_number:{} | event_number:{}",self.frame_number,self.event_number),

            WindowEvent::Occluded(occluded) => log::trace!("occluded - frame_number:{} | event_number:{} | occlusion:{}",self.frame_number,self.event_number,occluded),

            WindowEvent::Moved(_) => {
                if self.log_trace_window_move {
                    log::trace!("window moved - frame_number:{} | event_number:{}",self.frame_number,self.event_number);
                }
            },
            _ => {}
        };

        self.event_number += 1;
    }
}
