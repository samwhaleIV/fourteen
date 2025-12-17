use::wgpu::RenderPass;

use crate::wgpu::{
    frame::*,
    wgpu_handle::WGPUHandle,
    graphics_context::{
        GraphicsContext,
        GraphicsContextInternal
    },
    texture_container::SamplerMode
};

use generational_arena::Index;

pub fn render_frame(frame: &Frame,wgpu_handle: &impl WGPUHandle,context: &mut GraphicsContext) {
    /* This is not where the encoder is created. Only 1 encoder is created for the master, output frame. */
    if let Some(mut encoder) = context.try_borrow_encoder() {
        {
            let mut render_pass = context.create_render_pass(wgpu_handle,frame,&mut encoder);

            let queue = wgpu_handle.get_queue();
            process_commands(&mut render_pass,frame,context,queue);
        }
        context.return_encoder(encoder);
    } else {
        panic!("Graphics context did not provide an encoder!");
    }
}

fn process_commands(render_pass: &mut RenderPass,frame: &Frame,context: &mut GraphicsContext,queue: &wgpu::Queue) {

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
            let texture_container = context.get_texture_container(*new_index);

            if needs_sampler_update || match current_sampling_frame.take() {
                Some(current_index) => current_index != *new_index,
                None => true
            } {
                let sampler_mode = SamplerMode::get_mode(filter_mode,wrap_mode);
                let sampler = texture_container.get_bind_group(sampler_mode);
                render_pass.set_bind_group(GraphicsContext::TEXTURE_BIND_GROUP_INDEX,sampler,&[]);
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

            FrameCommand::DrawFrame(_,draw_data) => {
                context.write_quad(render_pass,queue,draw_data);
            },

            FrameCommand::DrawFrameSet(_,draw_data_set) => {
                context.write_quad_set(render_pass,queue,&draw_data_set);
            },
        }
    }
}
