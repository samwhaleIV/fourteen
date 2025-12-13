#![allow(dead_code,unused_variables)]

use wgpu::RenderPass;

use crate::{
    frame::{
        Frame,
        FrameCommand,
        FrameInternal,
    }, pipeline_management::Pipeline, wgpu_interface::WGPUInterface
};

pub fn render_frame(frame: &Frame,wgpu_interface: &impl WGPUInterface,pipeline: &mut Pipeline) {
    let device = wgpu_interface.get_device();

    if let Some(mut encoder) = pipeline.try_borrow_encoder() {

        //check if frame is output. then do stuff.

        let operations = wgpu::Operations {
            //TODO: Change these
            load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
            store: wgpu::StoreOp::Store,
        };
        
        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Output Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: wgpu_interface.get_output_texture(),
                depth_slice: None,
                resolve_target: None,
                ops: operations,
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        todo!();


        pipeline.return_encoder(encoder);
    } else {
        panic!("Pipeline did not provide an encoder.");
    }
}

fn process_commands(frame: &Frame,render_pass: &RenderPass) {
    /* Some deeply complex optimization option could coalesce commands together, but set commands should cover any optimization concerns. */
    for command in frame.get_command_buffer().iter() {
        match command {
            FrameCommand::DrawColor(value) => todo!(),
            FrameCommand::DrawFrame(finished_frame, value) => todo!(),
            FrameCommand::DrawFrameColored(finished_frame, value) => todo!(),
            
            FrameCommand::DrawColorSet(values) => todo!(),
            FrameCommand::DrawFrameSet(finished_frame,values) => todo!(),
            FrameCommand::DrawFrameColoredSet(finished_frame,values) => todo!(),

            FrameCommand::SetTextureFilter(filter_mode) => todo!(),
            FrameCommand::SetTextureWrap(wrap_mode) => todo!(),
        }
    }
}
