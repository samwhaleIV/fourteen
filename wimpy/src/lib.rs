#![allow(dead_code,unused_variables)]

pub mod frame;
pub mod area;
pub mod color;
pub mod texture;
mod frame_binder;

fn test() {

    let mut texture_cache = texture::TextureCache::default();
    let texture = texture_cache.load_texture_debug("Debug Texture");
    
    let frame = frame::create_frame(128,128);


}