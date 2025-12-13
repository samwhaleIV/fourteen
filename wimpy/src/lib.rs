#![allow(dead_code,unused_variables)]

use crate::{wgpu_interface::WGPUInterface, pipeline_management::Pipeline};

pub mod frame;
pub mod area;
pub mod color;
pub mod frame_cache;
pub mod lease_arena;
pub mod wgpu_interface;
pub mod texture_container;
mod pipeline_management;
mod frame_processor;

struct VirtualWGPUProvider {

}

impl WGPUInterface for VirtualWGPUProvider {
    fn get_device(&self) -> wgpu::Device {
        todo!()
    }

    fn get_queue(&self) -> wgpu::Queue {
        todo!()
    }

    fn get_output_format(&self) -> wgpu::TextureFormat {
        todo!()
    }

    fn get_pipeline(&self) -> &Pipeline {
        todo!()
    }

    fn get_output_size(&self) -> (u32,u32) {
        todo!()
    }

    fn get_output_texture(&self) -> wgpu::TextureView {
        todo!()
    }

    fn start_encoding(&mut self) {
        todo!()
    }

    fn get_encoder(&self) -> Option<&wgpu::CommandEncoder> {
        todo!()
    }

    fn finish_encoding(&mut self) {
        todo!()
    }
}

const MAX_QUADS: usize = 1000;
const MAX_UNIFORMS: usize = 100;

fn test() {

    let mut wgpu_interface = VirtualWGPUProvider {
        //This is where the magic binding happens. Pretend it is here already.
    };
    let mut pipeline = Pipeline::create(&wgpu_interface,MAX_QUADS,MAX_UNIFORMS);
    let mut cache = frame_cache::FrameCache::create();

    let texture_frame = cache.create_texture_frame_debug(&wgpu_interface);

    let mut output_frame = cache.get_output_frame(&wgpu_interface);

    wgpu_interface.start_encoding();


    output_frame.set_texture_filter(frame::FilterMode::Nearest);
    output_frame.set_texture_wrap(frame::WrapMode::Clamp);
    output_frame.finish(&mut cache,&mut pipeline,&wgpu_interface);


    wgpu_interface.finish_encoding();
}
