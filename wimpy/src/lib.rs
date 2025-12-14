//#![allow(dead_code,unused_variables)]

use image::ImageError;

use crate::{
    area::Area, color::Color, frame::DrawData, pipeline_management::Pipeline, wgpu_interface::WGPUInterface
};

mod frame;
mod area;
mod color;
mod lease_arena;
mod wgpu_interface;
mod texture_container;
mod pipeline_management;
mod frame_processor;

struct VirtualWGPUProvider {

}

impl WGPUInterface for VirtualWGPUProvider {
    fn get_device(&self) -> &wgpu::Device {
        todo!()
    }

    fn get_queue(&self) -> &wgpu::Queue {
        todo!()
    }

    fn get_output(&self) -> (wgpu::TextureView,(u32,u32)) {
        todo!()
    }

    fn get_output_format(&self) -> wgpu::TextureFormat {
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
    let mut pipeline = Pipeline::create_with_buffer_frames(
        &w,MAX_QUADS,MAX_UNIFORMS,&vec![(64,64)],4
    );

    let texture_frame = pipeline.load_texture(&w,"../../content/images/null.png")?;

    let mut f = pipeline.start(&mut w);

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

    return Ok(());
}
