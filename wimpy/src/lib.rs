#![allow(dead_code,unused_variables)]

use crate::{
    area::Area, frame::PositionUV, pipeline_management::{FrameCacheManagement, Pipeline}, wgpu_interface::WGPUInterface
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

    let mut w = VirtualWGPUProvider {
        //This is where the magic binding happens. Pretend it is here already.
    };
    let mut pipeline = Pipeline::create_with_buffer_frames(
        &w,MAX_QUADS,MAX_UNIFORMS,&vec![(64,64)],4
    );

    let texture_frame = pipeline.create_texture_frame_debug(&w);

    let mut output_frame = pipeline.start(&mut w);

    output_frame.set_texture_filter(frame::FilterMode::Nearest);
    output_frame.set_texture_wrap(frame::WrapMode::Clamp);

    output_frame.draw_frame(&texture_frame,PositionUV {
        position: Area::NORMAL,
        uv: Area::NORMAL,
    });
    

    output_frame.finish(&w,&mut pipeline);

    pipeline.finish(&mut w);

}
