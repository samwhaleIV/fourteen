#![allow(dead_code,unused_variables)]

use crate::frame_binder::WGPUInterface;

pub mod frame;
pub mod area;
pub mod color;
mod frame_binder;
mod pipeline_management;


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

    fn get_pipeline_manager(&self) -> &pipeline_management::PipelineManager {
        todo!()
    }

    fn get_output_size(&self) -> (u32,u32) {
        todo!()
    }

    fn get_output_texture(&self) -> wgpu::TextureView {
        todo!()
    }
}

fn test() {
    let wgpu_interface = VirtualWGPUProvider {};

    let mut frame_binder = frame_binder::FrameBinder::create();

    let texture = frame_binder.create_texture_frame_debug(&wgpu_interface);

    let output_frame = frame::Frame::create_output(&wgpu_interface);
}
