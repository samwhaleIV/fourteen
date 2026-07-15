use std::path::Path;

use wasm_bindgen::prelude::*;
use web_sys::{ImageBitmap, js_sys::{Object, Reflect, Uint8Array}};
use wgpu::{CopyExternalImageDestInfo, CopyExternalImageSourceInfo, Extent3d, ExternalImageSource, Origin2d, Origin3d, Queue, Texture};
use wimpy_engine::{UWimpyPoint, app::{FileError, WimpyIO, WimpyImageData, WimpyImageDataWriter}};

pub struct WimpyWebIO;

#[wasm_bindgen(module = "/html/wimpy-web-io.js")]
extern "C" {
    #[wasm_bindgen(js_name = saveKeyValueStore)]
    async fn save_key_value_store_js(data: Vec<u8>) -> JsValue;

    #[wasm_bindgen(js_name = saveKeyValueStore)]
    async fn load_key_value_store_js() -> JsValue;

    #[wasm_bindgen(js_name = loadTextFile)]
    async fn load_text_file_js(path: String) -> JsValue;

    #[wasm_bindgen(js_name = loadBinaryFile)]
    async fn load_binary_file_js(path: String) -> JsValue;

    #[wasm_bindgen(js_name = loadImageFile)]
    async fn load_image_file_js(path: String) -> JsValue;
}

fn get_js_file_function_result(value: JsValue) -> Result<JsValue,FileError> {
    let object = Object::from(value);

    match Reflect::get(&object,&"error".into()) {
        Ok(value) if !value.is_null_or_undefined() => return Err(match serde_wasm_bindgen::from_value::<FileError>(value) {
            Ok(file_error) => file_error,
            Err(_) => FileError::Unknown,
        }),
        Ok(_) => {}, // Property exists, but it has no value
        Err(_) => return Err(FileError::Internal) // An exception during the property 'get'
    };

    return match Reflect::get(&object,&"value".into()) {
        Ok(value) if !value.is_null_or_undefined() => Ok(value),
        Ok(_) => Err(FileError::Internal), // Property exists, but it has no value
        Err(_) => Err(FileError::Internal) // An exception during the property 'get'
    };
}

impl WimpyIO for WimpyWebIO {
    async fn save_key_value_store(data: &[u8]) -> Result<(),FileError> {
        _ = get_js_file_function_result(save_key_value_store_js(Vec::from(data)).await)?;
        Ok(())
    }

    async fn load_key_value_store() -> Result<Vec<u8>,FileError> {
        let js_value = get_js_file_function_result(load_key_value_store_js().await)?;
        if js_value.is_instance_of::<Uint8Array>() {
            let bytes = Uint8Array::from(js_value);
            Ok(bytes.to_vec())
        } else {
            Err(FileError::Internal)
        }
    }

    async fn load_binary_file(path: &Path) -> Result<Vec<u8>,FileError> {
        let path_str = path_to_str(path)?.to_string();
        let js_value = get_js_file_function_result(load_binary_file_js(path_str).await)?;
        if js_value.is_instance_of::<Uint8Array>() {
            let data = Uint8Array::from(js_value);
            Ok(data.to_vec())
        } else {
            Err(FileError::Internal)
        }
    }

    async fn load_text_file(path: &Path) -> Result<String,FileError> {
        let path_str = path_to_str(path)?.to_string();
        let js_value = get_js_file_function_result(load_text_file_js(path_str).await)?;
        match js_value.as_string() {
            Some(value) => Ok(value),
            None => Err(FileError::Internal),
        }
    }

    async fn load_image_file(path: &Path) -> Result<WimpyImageData<'_>,FileError> {
        let path_str = path_to_str(path)?.to_string();
        let js_value = get_js_file_function_result(load_image_file_js(path_str).await)?;
        if js_value.is_instance_of::<ImageBitmap>() {
            let image = ImageBitmap::from(js_value);
            let wrapper = ExternalImageSourceWrapper {
                value: ExternalImageSource::ImageBitmap(image),
            };
            let image_data = WimpyImageData::Custom { data: Box::new(wrapper) };
            Ok(image_data)
        } else {
            Err(FileError::Other)
        }
    }
}

struct ExternalImageSourceWrapper {
    value: ExternalImageSource
}

impl WimpyImageDataWriter for ExternalImageSourceWrapper {
    fn write(self: Box<Self>,queue: &Queue,texture: &Texture,max_size: UWimpyPoint) {
        let size = self.size();
        queue.copy_external_image_to_texture(
            &CopyExternalImageSourceInfo {
                source: self.value,
                origin: Origin2d::ZERO,
                flip_y: false,
            },
            CopyExternalImageDestInfo {
                texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
                color_space: wgpu::PredefinedColorSpace::Srgb,
                premultiplied_alpha: false,
            },
            Extent3d {
                width: size.x.min(max_size.x),
                height: size.y.max(max_size.y),
                depth_or_array_layers:1,
            }
        );
    }
    fn size(&self) -> UWimpyPoint {
        UWimpyPoint {
            x: self.value.width(),
            y: self.value.height()
        }
    }
}

fn path_to_str(path: &Path) -> Result<&str,FileError> {
    match path.to_str() {
        Some(value) => Ok(value),
        None => Err(FileError::InvalidPath),
    }
}

