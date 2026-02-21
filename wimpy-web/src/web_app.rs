use super::*;
use std::path::Path;

use std::{cell::RefCell,rc::Rc};
use wasm_bindgen::{JsCast,JsValue,prelude::Closure};
use web_sys::js_sys::Float32Array;
use web_sys::{Document, Event, HtmlCanvasElement, KeyboardEvent, Window};
use wgpu::{InstanceDescriptor,Limits,SurfaceTarget};

use wimpy_engine::{UWimpyPoint, WimpyRect, WimpyVec, app::*};
use wimpy_engine::app::graphics::*;
use wimpy_engine::app::input::*;
use wimpy_engine::shared::WimpyArea;

const CANVAS_ID: &'static str = "main-canvas";

/* Must match 'html/style.css @ div#virtual-cursor' */
const EMULATED_CURSOR_SIZE: UWimpyPoint = [12,16].into();

#[derive(Debug)]
pub enum WebAppError {
    WindowNotFound,
    DocumentNotFound,
    CanvasNotFound,
    InvalidCanvasElement,
    WGPUInitFailure,
    SurfaceCreationFailure,
    KeyEventBindFailure,
    RequestAnimationFrameFailure,
    ResizeEventBindFailure,
    PerformanceDoesNotExist,
    WimpyContextCreationFailure,
}

pub struct WebApp<TWimpyApp> {
    last_frame_time: f64,
    current_frame_time: f64,
    gamepad_manager: GamepadManager,
    size: UWimpyPoint,
    wimpy_app: TWimpyApp,
    wimpy_context: WimpyContext,
}

#[allow(unused)]
#[derive(PartialEq)]
pub enum ResizeConfig {
    Static,
    FitWindow,
}

#[wasm_bindgen(module = "/html/virtual-cursor.js")]
extern "C" {
    #[wasm_bindgen(js_name = updateVirtualCursor)]
    fn update_virtual_cursor(x: f32,y: f32,glyph: u8,is_emulated: bool,mode_switch_command: u8);

    #[wasm_bindgen(js_name = pollMouse)]
    fn poll_mouse_js() -> Float32Array;
}

fn poll_mouse() -> MouseInput {
    let src = poll_mouse_js();
    let mut buffer = [0.0f32;6];
    src.copy_to(&mut buffer);
    MouseInput {
        position: WimpyVec {
            x: buffer[0],
            y: buffer[1],
        },
        delta: WimpyVec {
            x: buffer[2],
            y: buffer[3],
        },
        left_pressed: buffer[4] != 0.0,
        right_pressed: buffer[5] != 0.0
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
        let gamepad_manager = GamepadManager::new();

        let instance = wgpu::Instance::new(&InstanceDescriptor {
            backends: wgpu::Backends::BROWSER_WEBGPU | wgpu::Backends::GL,
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

        let Some(mut wimpy_context) = WimpyContext::create::<WimpyWebIO,TConfig>(WimpyContextCreationConfig {
            manifest_path,
            input_type_hint: InputType::Unknown,
            graphics_provider,
        }).await else {
            return Err(WebAppError::WimpyContextCreationFailure);
        };

        let wimpy_app = TWimpyApp::load(&mut wimpy_context).await;

        return Ok(Rc::new(RefCell::new(Self {
            last_frame_time: 0.0,
            current_frame_time: 0.0,
            size: UWimpyPoint::ZERO,
            wimpy_context,
            wimpy_app,
            gamepad_manager,
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

        let mouse_input = poll_mouse();

        let delta_time = ((self.current_frame_time - self.last_frame_time) * 0.001) as f32;

        let mouse_shell_state = self.wimpy_context.input.update(
            mouse_input,
            gamepad_state,
            delta_time,
            WimpyRect {
                position: WimpyVec::ZERO,
                size: WimpyVec::from(self.size) - WimpyVec::from(EMULATED_CURSOR_SIZE)
            },
            false
        );

        update_virtual_cursor(
            mouse_shell_state.position.x,
            mouse_shell_state.position.y,
            match mouse_shell_state.cursor_glyph {
                CursorGlyph::None => 1,
                CursorGlyph::Default => 2,
                CursorGlyph::CanInteract => 3,
                CursorGlyph::IsInteracting => 4,
                CursorGlyph::CameraCrosshair => 5,
            },
            match mouse_shell_state.likely_active_device {
                LikelyActiveDevice::Mouse => false,
                LikelyActiveDevice::Gamepad => true,
            },
            match mouse_shell_state.mouse_mode {
                MouseMode::Interface => 1,
                MouseMode::Camera => 2,
            }
        );
    }

    fn render_frame(&mut self) {
        self.update_input();
        self.wimpy_app.update(&mut self.wimpy_context);
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

        let graphics_provider = self.wimpy_context.graphics.get_graphics_provider_mut();

        let inner_width = translate_html_size(window.inner_width());
        let inner_height = translate_html_size(window.inner_height());

        graphics_provider.set_size(
            inner_width,
            inner_height
        );

        let size = graphics_provider.get_size();

        canvas.set_width(size.x);
        canvas.set_height(size.y);

        self.size = size;
    }

    fn setup_events(app: &Rc<RefCell<Self>>,resize_config: ResizeConfig) -> Result<(),WebAppError> {
        {
            let app = app.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move|event: KeyboardEvent| {
                if event.repeat() {
                    return;
                }
                let Some(key_code) = KEY_CODES.get(&event.code()) else {
                    return;
                };
                app.borrow_mut().wimpy_context.input.set_key_code_pressed(*key_code);
            });
            get_document()?.add_event_listener_with_callback("keydown",closure.as_ref().unchecked_ref()).map_err(|_|WebAppError::KeyEventBindFailure)?;
            closure.forget();
        }
        {
            let app = app.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move|event: KeyboardEvent| {
                if event.repeat() {
                    return;
                }
                let Some(key_code) = KEY_CODES.get(&event.code()) else {
                    return;
                };
                app.borrow_mut().wimpy_context.input.set_key_code_released(*key_code);

            });
            get_document()?.add_event_listener_with_callback("keyup",closure.as_ref().unchecked_ref()).map_err(|_|WebAppError::KeyEventBindFailure)?;
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
