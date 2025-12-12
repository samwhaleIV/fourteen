use wgpu::{CommandEncoder, RenderPass};

use crate::{frame::{Frame, FrameCommand, FrameInternal, FrameType}, frame_cache::{FrameCache, WGPUInterface}};

pub fn render_frame(frame: &Frame,frame_cache: &FrameCache,wgpu_interface: &impl WGPUInterface) -> Frame {
    let device = wgpu_interface.get_device();

    if let Some(encoder) = wgpu_interface.get_encoder() {
        return match frame.get_type() {
            FrameType::Output => render_output_frame(frame,frame_cache,encoder),
            FrameType::Mutable => render_mutable_frame(frame,frame_cache,encoder),
            FrameType::Immutable => render_immutable_frame(frame,frame_cache,encoder),
            FrameType::Invalid => panic!("Can't render invalid frame. Another layer of validation should have prevented this panic."),
        };
    } else {
        log::error!("WGPU interface did not provide an encoder.");
        return Frame::create_null();
    }
}

fn render_output_frame(frame: &Frame,frame_cache: &FrameCache,encoder: &CommandEncoder) -> Frame {
    todo!();
}

fn render_mutable_frame(frame: &Frame,frame_cache: &FrameCache,encoder: &CommandEncoder) -> Frame {
    todo!();
}

fn render_immutable_frame(frame: &Frame,frame_cache: &FrameCache,encoder: &CommandEncoder) -> Frame {
    todo!();
}

fn process_commands(frame: &Frame,render_pass: &RenderPass) {
    /* Some deeply complex optimization option could coalesce commands together, but set commands should cover any optimization concerns. */
    for command in frame.get_command_buffer().iter() {
        match command {
            FrameCommand::DrawColor(position_color) => todo!(),
            FrameCommand::DrawFrame(finished_frame, position_uv) => todo!(),
            FrameCommand::DrawFrameColored(finished_frame, position_uvcolor) => todo!(),
            FrameCommand::DrawColorSet(position_colors) => todo!(),
            FrameCommand::DrawFrameSet(finished_frame, position_uvs) => todo!(),
            FrameCommand::DrawFrameColoredSet(finished_frame, position_uvcolors) => todo!(),
            FrameCommand::SetTextureFilter(filter_mode) => todo!(),
            FrameCommand::SetTextureWrap(wrap_mode) => todo!(),
        }
    }
}
