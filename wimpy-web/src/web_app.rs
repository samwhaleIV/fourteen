use std::{
    cell::RefCell,
    rc::Rc
};

use wasm_bindgen::{
    JsCast,
    JsValue,
    prelude::Closure
};

use wasm_bindgen_futures::JsFuture;

use web_sys::{
    Document,
    Event,
    HtmlCanvasElement,
    HtmlImageElement,
    ImageBitmap,
    KeyboardEvent,
    MouseEvent,
    Window
};

use wgpu::{
    Color, CopyExternalImageDestInfo, CopyExternalImageSourceInfo, ExternalImageSource, InstanceDescriptor, Limits, Origin2d, SurfaceTarget
};

use wimpy_engine::{
    WimpyApp,
    WimpyContext,
    WimpyIO,
    WimpyImageError,
    input::{
        GamepadInput, InputManager, InputManagerAppController, InputManagerReadonly
    },
    storage::{
        KeyValueStore,
        KeyValueStoreIO
    },
    wgpu::{
        GraphicsContext,
        GraphicsContextConfig,
        GraphicsContextController,
        GraphicsContextInternalController,
        GraphicsProvider,
        GraphicsProviderConfig,
        GraphicsProviderError,
        TextureData
    }
};

const CANVAS_ID: &'static str = "main-canvas";

use crate::key_code::KEY_CODES;

#[derive(Debug)]
pub enum WebAppError {
    WindowNotFound,
    DocumentNotFound,
    CanvasNotFound,
    InvalidCanvasElement,
    #[allow(unused)]
    WGPUInitFailure(GraphicsProviderError),
    SurfaceCreationFailure,
    MouseEventBindFailure,
    RequestAnimationFrameFailure,
    ResizeEventBindFailure,
}

pub struct WebApp<TWimpyApp,TConfig> {
    graphics_context: GraphicsContext<TConfig>,
    input_manager: InputManager,
    wimpy_app: TWimpyApp,
    key_value_store: KeyValueStore
}

#[allow(unused)]
#[derive(PartialEq)]
pub enum ResizeConfig {
    Static,
    FitWindow
}

pub struct WebAppIO;

struct ExternalImageSourceWrapper {
    value: ExternalImageSource
}

impl TextureData for ExternalImageSourceWrapper {
    fn size(&self) -> (u32,u32) {
        return (self.value.width(),self.value.height());
    }
    
    fn write_to_queue(self,parameters: &wimpy_engine::wgpu::TextureDataWriteParameters) {
        parameters.queue.copy_external_image_to_texture(
            &CopyExternalImageSourceInfo {
                source: self.value,
                origin: Origin2d::ZERO,
                flip_y: false,
            },
            CopyExternalImageDestInfo {
                texture: parameters.texture,
                mip_level: parameters.mip_level,
                origin: parameters.origin,
                aspect: parameters.aspect,
                color_space: wgpu::PredefinedColorSpace::Srgb,
                premultiplied_alpha: false,
            },
            parameters.texture_size
        );
    }
}

impl WimpyIO for WebAppIO {
    fn save_key_value_store(kvs: &KeyValueStore) {
        let data = kvs.export();
        todo!()
    }

    fn load_key_value_store(kvs: &mut KeyValueStore) {
        let data = todo!();
        kvs.import(data);
    }

    async fn get_image(path: &'static str) -> Result<impl TextureData,WimpyImageError> {
        let window = get_window().expect("window exists");

        let image_element = HtmlImageElement::new().expect("html image element creation");
        image_element.set_src(path);

        let Ok(bitmap) = get_image_bitmap(&window,&image_element).await else {
            log::error!("Could not create ImageBitmap for '{}'.",path);
            return Err(WimpyImageError::Unknown); 
        };

        return Ok(ExternalImageSourceWrapper {
            value: ExternalImageSource::ImageBitmap(bitmap),
        });
    }
}

async fn get_image_bitmap(window: &Window,image_element: &HtmlImageElement) -> Result<ImageBitmap,JsValue> {
    Ok(JsFuture::from(window.create_image_bitmap_with_html_image_element(image_element)?).await?.dyn_into::<ImageBitmap>()?)
}

impl<TWimpyApp,TConfig> WebApp<TWimpyApp,TConfig>
where
    TWimpyApp: WimpyApp<WebAppIO,TConfig> + 'static,
    TConfig: GraphicsContextConfig + 'static
{
    pub async fn create_app(wimpy_app: TWimpyApp) -> Result<Rc<RefCell<Self>>,WebAppError> {
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
            Err(error) => Err(WebAppError::WGPUInitFailure(error)),
        }?;

        let graphics_context = GraphicsContext::<TConfig>::create(graphics_provider);

        let input_manager = InputManager::default();

        return Ok(Rc::new(RefCell::new(Self {
            graphics_context,
            input_manager,
            wimpy_app,
            key_value_store: Default::default(),
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
        //TODO
    }

    fn mouse_up(&mut self,x: i32,y: i32) {
        //TODO
    }

    fn mouse_move(&mut self,x: i32,y: i32) {
        //TODO
    }

    fn render_frame(&mut self) {
        self.input_manager.update(GamepadInput::default());

        let axes = self.input_manager.get_axes();

        log::info!("GAMEPAD BS: Y Axis: {:?}",axes.y);

        let mut output_frame = match self.graphics_context.create_output_frame(Color::RED) {
            Ok(value) => value,
            Err(error) => {
                log::error!("Could not create output frame: {:?}",error);
                return;
            }
        };

        let app_context = WimpyContext {
            graphics: &mut self.graphics_context,
            storage: &mut self.key_value_store,
            input: &mut self.input_manager,
        };

        self.wimpy_app.render(&app_context);

        if let Err(error) = self.graphics_context.render_frame(&mut output_frame) {
            log::error!("{:?}",error);
        }

        if let Err(error) = self.graphics_context.present_output_frame(output_frame) {
            log::error!("{:?}",error);
        }
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

        graphics_provider.set_size(
            translate_html_size(window.inner_width()),
            translate_html_size(window.inner_height())
        );

        let (width,height) = graphics_provider.get_size();

        canvas.set_width(width);
        canvas.set_height(height);

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
