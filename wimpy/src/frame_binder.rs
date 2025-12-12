use std::collections::{HashMap, HashSet};

use generational_arena::Arena;
use image::{DynamicImage, ImageError, ImageReader};

use crate::frame::{FrameInternal, Frame, FrameCommand};
use crate::pipeline_management::{PipelineManager, TextureContainer};

pub struct FrameBinder {
    textures: Arena<TextureContainer>,
    mutable_textures: HashMap<(u32,u32),generational_arena::Index>,
    leased_mutable_textures: HashSet<generational_arena::Index>
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
    pub fn create() -> Self {
        return Self {
            textures: Arena::default(),
            mutable_textures: HashMap::default(),
            leased_mutable_textures: HashSet::default()
        }
    }
    pub fn create_with_buffer_frames(sizes: &[(u32,u32)],wgpu_interface: &impl WGPUInterface) -> Self {

        let capacity = sizes.len();

        let mut textures = Arena::with_capacity(capacity);
        let mut mutable_textures = HashMap::with_capacity(capacity);

        for size in sizes.iter() {
            let mutable_texture = TextureContainer::create_mutable(*size,wgpu_interface);
            let index = textures.insert(mutable_texture);
            mutable_textures.insert(*size,index);
        }

        return Self {
            textures,
            mutable_textures,
            leased_mutable_textures: HashSet::default()
        }
    }
}

impl FrameBinder {
    fn request_mutable_texture(size: (u32,u32)) -> Option<TextureContainer> {
        //todo
        return None;
    }
    pub fn render_frame(&self,frame: &Frame,wgpu_interface: &impl WGPUInterface) -> Frame {
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

    fn create_finished_frame(&mut self,image: &DynamicImage,wgpu_interface: &impl WGPUInterface) -> Frame {
        let texture_container = TextureContainer::from_image(&image,wgpu_interface);
        let size = texture_container.size();
        let index = self.textures.insert(texture_container);
        return Frame::to_immutable(size,index);
    }

    pub fn create_texture_frame(&mut self,name: &str,wgpu_interface: &impl WGPUInterface) -> Result<Frame,ImageError> {
        let image = ImageReader::open(name)?.decode()?;
        let frame = self.create_finished_frame(&image,wgpu_interface);
        return Ok(frame);
    }

    pub fn create_texture_frame_debug(&mut self,wgpu_interface: &impl WGPUInterface) -> Frame {
        let bytes = include_bytes!("../../content/images/null.png");
        let image = image::load_from_memory(bytes).unwrap();
        let frame = self.create_finished_frame(&image,wgpu_interface);
        return frame;
    }
}
