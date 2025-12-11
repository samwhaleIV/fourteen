#![allow(dead_code,unused_variables)]

use collections::named_cache::{CacheItemReference,NamedCache};
use image::{DynamicImage,ImageError,ImageReader};

#[derive(Default)]
pub struct TextureCache {
    cache: NamedCache<DynamicImage>
}

#[derive(Copy,Clone)]
pub struct Texture {
    reference: CacheItemReference
}

impl TextureCache {
    pub fn load_texture(&mut self,name: &str) -> Result<Texture,ImageError> {
        if let Some(reference) = self.cache.get_reference(name) {
            return Ok(Texture { reference });
        }

        let image = ImageReader::open(name)?.decode()?;

        let reference = self.cache.store_item(name,image);

        return Ok(Texture { reference });
    }

    pub fn load_texture_debug(&mut self,name: &str) -> Texture {
        let bytes = include_bytes!("../../content/images/test_image.png");
        let image = image::load_from_memory(bytes).unwrap();

        let reference = self.cache.store_item(name,image);
        return Texture { reference };
    }
    
    pub fn unload_texture(&mut self,texture: &Texture) {
        self.cache.remove_item(&texture.reference);
    }

    pub fn borrow_texture(&self,texture: &Texture) -> &DynamicImage {
        return self.cache.borrow_item(&texture.reference);
    }
}
