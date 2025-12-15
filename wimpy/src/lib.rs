//#![allow(dead_code,unused_variables)]

use image::ImageError;

use crate::{
    area::Area, color::Color, frame::DrawData, pipeline_management::{CacheOptions, Pipeline, PipelineCreationOptions}, wgpu_interface::WGPUInterface
};

mod frame;
mod lease_arena;
mod texture_container;
mod frame_processor;

pub mod color;
pub mod area;

pub mod pipeline_management;
pub mod wgpu_interface;

struct VirtualWGPUProvider {

}

impl WGPUInterface for VirtualWGPUProvider {
    fn get_device(&self) -> &wgpu::Device {
        todo!()
    }

    fn get_queue(&self) -> &wgpu::Queue {
        todo!()
    }

    fn get_output_format(&self) -> wgpu::TextureFormat {
        todo!()
    }
    
    fn get_output_surface(&self) -> Option<wgpu::SurfaceTexture> {
        todo!()
    }
}

const MAX_QUADS: u32 = 1000;
const MAX_UNIFORMS: u32 = 100;

#[allow(dead_code)]
fn test() -> Result<(),ImageError> {

    let mut w = VirtualWGPUProvider {
        //This is where the magic binding happens. Pretend it is here already.
    };

    let mut pipeline = Pipeline::create(&w,PipelineCreationOptions {
        quad_instance_capacity: MAX_QUADS,
        uniform_capacity: MAX_UNIFORMS,
        cache_options: Some(CacheOptions { instances: 4, sizes: vec![(64,64)] })
    });

    let texture_frame = pipeline.load_texture(&w,"../../content/images/null.png")?;

    if let Some(f) = &mut pipeline.start(&mut w) {
        f.set_texture_filter(frame::FilterMode::Nearest);
        f.set_texture_wrap(frame::WrapMode::Clamp);

        f.draw_frame(&texture_frame,DrawData {
            area: Area::one(),
            uv: Area::one(),
            rotation: 0.0,
            color: Color::BLACK,
        });

        f.finish(&w,&mut pipeline);

        pipeline.finish(&mut w);
    }

    return Ok(());
}
