use generational_arena::Arena;
use image::{DynamicImage, ImageError, ImageReader};
use wgpu::TextureView;

use crate::frame::{FinishedFrame, FinishedFrameInternal, Frame, FrameCommand, FrameInternal};
use crate::pipeline_management::{PipelineManager, TextureContainer};

pub struct FrameBinder {
    frames: Arena<TextureContainer>
}
pub trait WGPUInterface {
    fn get_device(&self) -> wgpu::Device;
    fn get_queue(&self) -> wgpu::Queue;
    fn get_output_format(&self) -> wgpu::TextureFormat;
    fn get_pipeline_manager(&self) -> &PipelineManager;
    fn get_output_size(&self) -> (u32,u32);
    fn get_output_texture(&self) -> wgpu::TextureView;
}

impl FrameBinder {
    pub fn render_frame(&mut self,frame: &Frame,wgpu_interface: &impl WGPUInterface) -> FinishedFrame {
        //TODO: Do stuff with FrameUsage

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

        panic!();
    }

    fn create_finished_frame(&mut self,image: &DynamicImage,wgpu_interface: &impl WGPUInterface) -> FinishedFrame {
        let texture_container = TextureContainer::from_image(&image,wgpu_interface);
        let size = texture_container.size();
        let index = self.frames.insert(texture_container);
        return FinishedFrame::create_immutable(size,index);
    }

    pub fn create_texture_frame(&mut self,name: &str,wgpu_interface: &impl WGPUInterface) -> Result<FinishedFrame,ImageError> {
        let image = ImageReader::open(name)?.decode()?;
        let frame = self.create_finished_frame(&image,wgpu_interface);
        return Ok(frame);
    }

    pub fn create_texture_frame_debug(&mut self,wgpu_interface: &impl WGPUInterface) -> FinishedFrame {
        let bytes = include_bytes!("../../content/images/null.png");
        let image = image::load_from_memory(bytes).unwrap();
        let frame = self.create_finished_frame(&image,wgpu_interface);
        return frame;
    }
}
