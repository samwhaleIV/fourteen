use std::{
    cell::RefCell,
    rc::Rc
};

use wasm_bindgen::{
    JsCast,
    JsValue,
    prelude::Closure
};

use web_sys::{
    Document,
    Event,
    HtmlCanvasElement,
    KeyboardEvent,
    MouseEvent,
    Window
};

use wgpu::{
    InstanceDescriptor, Limits, SurfaceTarget
};

use wimpy_engine::{
    WimpyAppHandler,
    input::InputManager,
    wgpu::{
        GraphicsContext, GraphicsContextConfig, GraphicsProvider, GraphicsProviderConfig, GraphicsProviderError, validate_surface_dimension
    }
};

#[derive(Debug)]
pub enum WebAppError {
    WindowNotFound,
    DocumentNotFound,
    CanvasNotFound,
    InvalidCanvasElement,
    WGPUInitFailure(GraphicsProviderError),
    SurfaceCreationFailure,
    MouseEventBindFailure,
    RequestAnimationFrameFailure,
    ResizeEventBindFailure
}

const CANVAS_ID: &'static str = "main-canvas";

pub struct WebApp<TWimpyApp,TConfig> {
    graphics_context: GraphicsContext<TConfig>,
    input_manager: InputManager,
    wimpy_app: TWimpyApp,
}

#[derive(PartialEq)]
pub enum ResizeConfig {
    Static,
    FitWindow
}

impl<TWimpyApp,TConfig> WebApp<TWimpyApp,TConfig>
where
    TWimpyApp: WimpyAppHandler + 'static,
    TConfig: GraphicsContextConfig + 'static
{
    pub async fn create_app(wimpy_app: TWimpyApp) -> Result<Rc<RefCell<Self>>,WebAppError> {
        let canvas = get_canvas()?;

        let (width,height) = (canvas.width(),canvas.height());

        let instance = wgpu::Instance::new(&InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..InstanceDescriptor::default()
        });
        let surface_target = SurfaceTarget::Canvas(canvas);
        let surface = match instance.create_surface(surface_target) {
            Ok(surface) => surface,
            Err(_) => return Err(WebAppError::SurfaceCreationFailure),
        };

        let graphics_context = GraphicsContext::<TConfig>::create(match GraphicsProvider::new(GraphicsProviderConfig {
            limits: Limits::downlevel_webgl2_defaults(),
            instance,
            surface,
            width,
            height,
        }).await {
            Ok(value) => value,
            Err(error) => return Err(WebAppError::WGPUInitFailure(error)),
        });

        let input_manager = InputManager::default();

        return Ok(Rc::new(RefCell::new(Self {
            graphics_context,
            input_manager,
            wimpy_app,
        })));
    }

    pub fn start_render_loop(app: Rc<RefCell<Self>>) -> Result<(),WebAppError> {
        let f = Rc::new(RefCell::new(None));
        let g = f.clone();
        *g.borrow_mut() = Some(Closure::new(move || {
            app.borrow_mut().render_frame();
            if let Err(error) = request_animation_frame(f.borrow().as_ref().unwrap()) {
                log::error!("{:?}",error);
            }
        }));
        request_animation_frame(g.borrow().as_ref().unwrap())?;
        return Ok(());
    }

    pub async fn run(wimpy_app: TWimpyApp,resize_config: ResizeConfig) -> Result<(),WebAppError> {
        let app = Self::create_app(wimpy_app).await?;
        app.borrow_mut().update_size();
        Self::setup_events(&app,resize_config)?;
        Self::start_render_loop(app.clone())?;
        return Ok(());
    }

    fn mouse_down(&mut self,x: i32,y: i32) {
        log::trace!("Web app: Mouse Down - ({},{})",x,y);
    }

    fn mouse_up(&mut self,x: i32,y: i32) {
        log::trace!("Web app: Mouse Up - ({},{})",x,y);
    }

    fn mouse_move(&mut self,x: i32,y: i32) {
        log::trace!("Web app: Mouse Move - ({},{})",x,y);
    }

    fn render_frame(&mut self) {
        //log::trace!("Mouse Down - ({},{})",x,y);
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

        let width = get_safe_canvas_dimension(window.inner_width());
        let height = get_safe_canvas_dimension(window.inner_height());

        canvas.set_width(width);
        canvas.set_height(height);

        self.graphics_context.get_graphics_provider_mut().set_size(width,height);

        log::trace!("Web app: Update Size - ({},{})",width,height);
    }

    fn setup_events(app: &Rc<RefCell<Self>>,resize_config: ResizeConfig) -> Result<(),WebAppError> {
        {
            let app = app.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move|event: MouseEvent| {
                if event.button() != 0 {
                    return;
                }
                app.borrow_mut().mouse_down(event.offset_x(),event.offset_y());
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
                app.borrow_mut().mouse_up(event.offset_x(),event.offset_y());
            });
            get_document()?.add_event_listener_with_callback("mouseup",closure.as_ref().unchecked_ref()).map_err(|_|WebAppError::MouseEventBindFailure)?;
            closure.forget();
        }
        {
            let app = app.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move|event: MouseEvent| {
                app.borrow_mut().mouse_move(event.offset_x(),event.offset_y());
            });
            get_document()?.add_event_listener_with_callback("mousemove",closure.as_ref().unchecked_ref()).map_err(|_|WebAppError::MouseEventBindFailure)?;
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

fn get_safe_canvas_dimension(value: Result<::wasm_bindgen::JsValue, JsValue>) -> u32 {
    validate_surface_dimension(value
        .unwrap_or(JsValue::from_f64(0.0))
        .as_f64()
        .unwrap_or(0.0) as u32
    )
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

fn get_canvas() -> Result<HtmlCanvasElement,WebAppError> {
    match get_document()?.get_element_by_id(CANVAS_ID) {
        Some(element) => match element.dyn_into::<HtmlCanvasElement>() {
            Ok(canvas) => Ok(canvas),
            Err(_) => Err(WebAppError::InvalidCanvasElement),
        },
        None => Err(WebAppError::CanvasNotFound),
    }
}
