#![allow(dead_code,unused_variables)]

use generational_arena::Index;
use wgpu::RenderPass;

use crate::{
    frame::{
        FilterMode,
        Frame,
        FrameCommand,
        FrameInternal,
        WrapMode
    },
    pipeline_management::Pipeline,
    texture_container::SamplerMode,
    wgpu_interface::WGPUInterface
};

pub fn render_frame(frame: &Frame,wgpu_interface: &impl WGPUInterface,pipeline: &mut Pipeline) {
    /* This is not where the encoder is created. Only 1 encoder is created for the master, output frame. */
    if let Some(mut encoder) = pipeline.try_borrow_encoder() {
        {
            let operations = wgpu::Operations {
                load: match frame.get_clear_color() {
                    Some(color) => wgpu::LoadOp::Clear(color),
                    None => wgpu::LoadOp::Load,
                },
                store: wgpu::StoreOp::Store,
            };

            let view = pipeline.get_texture_container(frame.get_index()).get_view();

            let mut render_pass = pipeline.config_render_pass(
                wgpu_interface,
                frame.size(),
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: operations,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            }));

            process_commands(frame,&mut render_pass,pipeline,wgpu_interface.get_queue());
        }
        pipeline.return_encoder(encoder);
    } else {
        panic!("Console did not provide an encoder!");
    }
}

fn process_commands(frame: &Frame,render_pass: &mut RenderPass,pipeline: &mut Pipeline,queue: &wgpu::Queue) {

    let mut needs_sampler_update: bool = true;

    let mut filter_mode: FilterMode = FilterMode::Nearest;
    let mut wrap_mode: WrapMode = WrapMode::Clamp;

    let mut current_sampling_frame: Option<Index> = None;

    /* Some deeply complex optimization option could coalesce commands together, but set commands should cover any optimization concerns. */
    for command in frame.get_command_buffer().iter() {

        if let Some(new_index) = match command {
            FrameCommand::DrawFrame(index,_) |
            FrameCommand::DrawFrameSet(index,_) => Some(index),
            //Add more types if they change the sampler bind group
            _ => None
        } {
            let texture_container = pipeline.get_texture_container(*new_index);

            if needs_sampler_update || match current_sampling_frame.take() {
                Some(current_index) => current_index != *new_index,
                None => true
            } {
                let sampler_mode = SamplerMode::get_mode(filter_mode,wrap_mode);
                let sampler = texture_container.get_bind_group(sampler_mode);
                render_pass.set_bind_group(Pipeline::TEXTURE_BIND_GROUP_INDEX,sampler,&[]);
            }
            needs_sampler_update = false;
            current_sampling_frame = Some(*new_index);
        }

        match command {
            FrameCommand::SetTextureFilter(value) => {
                if filter_mode != *value {
                    filter_mode = *value;
                    needs_sampler_update = true;
                }
            },

            FrameCommand::SetTextureWrap(value) => {
                if wrap_mode != *value {
                    wrap_mode = *value;
                    needs_sampler_update = true;
                }
            },

            FrameCommand::DrawFrame(_,draw_data) => pipeline.write_quad(queue,draw_data),

            FrameCommand::DrawFrameSet(_,draw_data_set) => pipeline.write_quad_set(queue,&draw_data_set),
        }
    }
}
