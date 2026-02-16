use super::*;
use std::path::Path;

use std::{cell::RefCell,rc::Rc};
use wasm_bindgen::{JsCast,JsValue,prelude::Closure};
use web_sys::{Document, Event, HtmlCanvasElement, KeyboardEvent, MouseEvent, Performance, Window};
use wgpu::{InstanceDescriptor,Limits,SurfaceTarget};

use wimpy_engine::app::*;
use wimpy_engine::app::graphics::*;
use wimpy_engine::app::input::*;
use wimpy_engine::app::wam::*;
use wimpy_engine::shared::WimpyArea;

const CANVAS_ID: &'static str = "main-canvas";
const LEFT_MOUSE_BUTTON: i16 = 0;
const RIGHT_MOUSE_BUTTON: i16 = 2;

#[derive(Debug)]
pub enum WebAppError {
    WindowNotFound,
    DocumentNotFound,
    CanvasNotFound,
    InvalidCanvasElement,
    WGPUInitFailure,
    SurfaceCreationFailure,
    MouseEventBindFailure,
    RequestAnimationFrameFailure,
    ResizeEventBindFailure,
    PerformanceDoesNotExist
}

pub struct WebApp<TWimpyApp> {
    graphics_context: GraphicsContext,
    input_manager: InputManager,
    wimpy_app: TWimpyApp,
    asset_manager: AssetManager,
    gamepad_manager: GamepadManager,
    kvs_store: KeyValueStore,
    mouse_state: MouseState,
    last_frame_time: f64,
    current_frame_time: f64,
    size: (u32,u32)
}

#[allow(unused)]
#[derive(PartialEq)]
pub enum ResizeConfig {
    Static,
    FitWindow,
}

struct MouseState {
    last_x: i32,
    last_y: i32,
    x: i32,
    y: i32,
    left_pressed: bool,
    right_pressed: bool
}

impl MouseState {
    pub fn to_wimpy_mouse_state(&self) -> MouseInput {
        return MouseInput {
            position:  MousePosition {
                x: self.x as f32,
                y: self.y as f32
            },
            delta: MouseDelta {
                x: (self.last_x - self.x) as f32,
                y: (self.last_y - self.y) as f32
            },
            left_pressed: self.left_pressed,
            right_pressed: self.right_pressed
        }
    }
}

impl Default for MouseState {
    fn default() -> Self {
        Self {
            last_x: 0,
            last_y: 0,
            x: 0,
            y: 0,
            left_pressed: false,
            right_pressed: false
        }
    }
}

impl<TWimpyApp> WebApp<TWimpyApp>
where
    TWimpyApp: WimpyApp<WimpyWebIO> + 'static,
{
    pub async fn create_app<TConfig>(manifest_path: Option<&Path>) -> Result<Rc<RefCell<Self>>,WebAppError>
    where
        TConfig: GraphicsContextConfig
    {
        let canvas = get_canvas()?;

        let instance = wgpu::Instance::new(&InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..InstanceDescriptor::default()
        });
        let surface_target = SurfaceTarget::Canvas(canvas);
        let surface = match instance.create_surface(surface_target) {
            Ok(surface) => surface,
            Err(_) => return Err(WebAppError::SurfaceCreationFailure),
        };

        let graphics_provider = match GraphicsProvider::new(GraphicsProviderConfig {
            limits: Limits::downlevel_webgl2_defaults(),
            instance,
            surface,
        }).await {
            Ok(value) => Ok(value),
            Err(error) => {
                log::error!("Graphics provider error: {:?}",error);
                return Err(WebAppError::WGPUInitFailure);
            },
        }?;

        let mut graphics_context = GraphicsContext::create::<TConfig>(graphics_provider);
        let mut input_manager = InputManager::default();
        let mut kvs_store = KeyValueStore::default();

        let gamepad_manager = GamepadManager::new();

        let mut asset_manager = AssetManager::load_or_default::<WimpyWebIO>(manifest_path).await;

        let wimpy_app = TWimpyApp::load(&mut WimpyContext {
            graphics: &mut graphics_context,
            storage: &mut kvs_store,
            input: &mut input_manager,
            assets: &mut asset_manager
        }).await;

        return Ok(Rc::new(RefCell::new(Self {
            last_frame_time: 0.0,
            current_frame_time: 0.0,
            size: (0,0),
            gamepad_manager,
            graphics_context,
            input_manager,
            asset_manager,
            wimpy_app,
            mouse_state: Default::default(),
            kvs_store: Default::default(),
        })));
    }

    pub fn start_render_loop(app: Rc<RefCell<Self>>) -> Result<(),WebAppError> {
        let f = Rc::new(RefCell::new(None));
        let g = f.clone();
        *g.borrow_mut() = Some(Closure::new(move || {
            let mut app_ref = app.borrow_mut();
            let now = get_performance_now_time();
            app_ref.last_frame_time = app_ref.current_frame_time;
            app_ref.current_frame_time = now;
            app_ref.render_frame();
            if let Err(error) = request_animation_frame(f.borrow().as_ref().unwrap()) {
                log::error!("{:?}",error);
            }
        }));
        request_animation_frame(g.borrow().as_ref().unwrap())?;
        return Ok(());
    }

    pub async fn run<TConfig>(manifest_path: Option<&Path>,resize_config: ResizeConfig) -> Result<(),WebAppError>
    where
        TConfig: GraphicsContextConfig
    {
        let app = Self::create_app::<TConfig>(manifest_path).await?;
        app.borrow_mut().update_size();
        Self::setup_events(&app,resize_config)?;
        Self::start_render_loop(app.clone())?;
        return Ok(());
    }

    fn update_input(&mut self) {
        self.gamepad_manager.update();
        let gamepad_state = create_gamepad_state(
            self.gamepad_manager.buffer()
        );

        let mouse_input = self.mouse_state.to_wimpy_mouse_state();

        self.mouse_state.last_x = self.mouse_state.x;
        self.mouse_state.last_y = self.mouse_state.y;

        let delta_time = ((self.current_frame_time - self.last_frame_time) * 0.001) as f32;

        let mouse_shell_state = self.input_manager.update(
            mouse_input,
            gamepad_state,
            delta_time,
            WimpyArea {
                x: 0.0,
                y: 0.0,
                width: self.size.0 as f32,
                height: self.size.1 as f32
            }
        );
        //log::trace!("Virtual Mouse Position: {:?}",self.input_manager.get_virtual_mouse().get_position());
    }

    fn render_frame(&mut self) {
        self.update_input();

        self.wimpy_app.update(&mut WimpyContext {
            graphics: &mut self.graphics_context,
            storage: &mut self.kvs_store,
            input: &mut self.input_manager,
            assets: &mut self.asset_manager
        });
    }

    fn key_down(&mut self,code: String) {
        let Some(key_code) = KEY_CODES.get(&code) else {
            return;
        };
        self.input_manager.set_key_code_pressed(*key_code);
    }

    fn key_up(&mut self,code: String) {
        let Some(key_code) = KEY_CODES.get(&code) else {
            return;
        };
        self.input_manager.set_key_code_released(*key_code);
    }

    fn update_size(&mut self) {
        let Ok(window) = get_window() else {
            log::error!("Web app: Window does not exist!");
            return;
        };

        let Ok(canvas) = get_canvas() else {
            log::error!("Web app: Canvas does not exist!");
            return;
        };

        let graphics_provider = self.graphics_context.get_graphics_provider_mut();

        let inner_width = translate_html_size(window.inner_width());
        let inner_height = translate_html_size(window.inner_height());

        graphics_provider.set_size(
            inner_width,
            inner_height
        );

        let (width,height) = graphics_provider.get_size();

        canvas.set_width(width);
        canvas.set_height(height);

        self.size = (inner_width,inner_height);
        //log::trace!("Web app: Update Size - ({},{})",width,height);
    }

    fn setup_events(app: &Rc<RefCell<Self>>,resize_config: ResizeConfig) -> Result<(),WebAppError> {
        {
            let app = app.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move|event: MouseEvent| {
                match event.button() {
                    LEFT_MOUSE_BUTTON => {
                        let mouse_state = &mut app.borrow_mut().mouse_state;
                        mouse_state.left_pressed = true;
                        mouse_state.x = event.client_x();
                        mouse_state.y = event.client_y();
                    },
                    RIGHT_MOUSE_BUTTON => {
                        let mouse_state = &mut app.borrow_mut().mouse_state;
                        mouse_state.right_pressed = true;
                        mouse_state.x = event.client_x();
                        mouse_state.y = event.client_y();
                    },
                    _ => {}
                }
            });
            get_document()?.add_event_listener_with_callback("mousedown",closure.as_ref().unchecked_ref()).map_err(|_|WebAppError::MouseEventBindFailure)?;
            closure.forget();
        }
        {
            let app = app.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move|event: MouseEvent| {
                if event.button() != 0 {
                    return;
                }
                match event.button() {
                    LEFT_MOUSE_BUTTON => {
                        let mouse_state = &mut app.borrow_mut().mouse_state;
                        mouse_state.left_pressed = false;
                        mouse_state.x = event.client_x();
                        mouse_state.y = event.client_y();
                    },
                    RIGHT_MOUSE_BUTTON => {
                        let mouse_state = &mut app.borrow_mut().mouse_state;
                        mouse_state.right_pressed = false;
                        mouse_state.x = event.client_x();
                        mouse_state.y = event.client_y();
                    },
                    _ => {}
                }
            });
            get_document()?.add_event_listener_with_callback("mouseup",closure.as_ref().unchecked_ref()).map_err(|_|WebAppError::MouseEventBindFailure)?;
            closure.forget();
        }
        {
            let app = app.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move|event: MouseEvent| {
                let mouse_state = &mut app.borrow_mut().mouse_state;
                mouse_state.x = event.client_x();
                mouse_state.y = event.client_y();
            });
            get_document()?.add_event_listener_with_callback("mousemove",closure.as_ref().unchecked_ref()).map_err(|_|WebAppError::MouseEventBindFailure)?;
            closure.forget();
        }
        {
            let app = app.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move|event: KeyboardEvent| {
                if event.repeat() {
                    return;
                }
                app.borrow_mut().key_down(event.code());
            });
            get_document()?.add_event_listener_with_callback("keydown",closure.as_ref().unchecked_ref()).map_err(|_|WebAppError::MouseEventBindFailure)?;
            closure.forget();
        }
        {
            let app = app.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move|event: KeyboardEvent| {
                if event.repeat() {
                    return;
                }
                app.borrow_mut().key_up(event.code());
            });
            get_document()?.add_event_listener_with_callback("keyup",closure.as_ref().unchecked_ref()).map_err(|_|WebAppError::MouseEventBindFailure)?;
            closure.forget();
        }
        if resize_config == ResizeConfig::FitWindow {
            let app = app.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move|_: Event| {
                app.borrow_mut().update_size();
            });
            get_window()?.add_event_listener_with_callback("resize",closure.as_ref().unchecked_ref()).map_err(|_|WebAppError::ResizeEventBindFailure)?;
            closure.forget();
        }
        return Ok(());
    }
}

fn get_window() -> Result<Window,WebAppError> {
    return web_sys::window().ok_or(WebAppError::WindowNotFound);
}

fn get_document() -> Result<Document,WebAppError> {
    let window = get_window()?;
    return window.document().ok_or(WebAppError::DocumentNotFound);
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) -> Result<(),WebAppError> {
    match get_window()?.request_animation_frame(f.as_ref().unchecked_ref()) {
        Ok(_) => Ok(()),
        Err(_) => Err(WebAppError::RequestAnimationFrameFailure)
    }
}

fn get_performance_now_time() -> f64 {
    let Ok(window) = get_window() else {
        log::warn!("Window not found! Cannot get performance.now() time! The world has stopped!");
        return 0.0;
    };
    match window.performance() {
        Some(value) => value.now(),
        None => {
            log::warn!("Cannot get time from performance.now()! Did someone in JavaScript land break it?");
            return 0.0;
        }
    }
}

fn get_canvas() -> Result<HtmlCanvasElement,WebAppError> {
    match get_document()?.get_element_by_id(CANVAS_ID) {
        Some(element) => match element.dyn_into::<HtmlCanvasElement>() {
            Ok(canvas) => Ok(canvas),
            Err(_) => Err(WebAppError::InvalidCanvasElement),
        },
        None => Err(WebAppError::CanvasNotFound),
    }
}

fn translate_html_size(value: Result<::wasm_bindgen::JsValue,JsValue>) -> u32 {
    value.unwrap_or(JsValue::from_f64(0.0)).as_f64().unwrap_or(0.0) as u32
}
