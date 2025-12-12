use std::collections::{HashMap, HashSet, VecDeque};

use generational_arena::Arena;
use image::{DynamicImage, ImageError, ImageReader};

use crate::frame::{FrameInternal, Frame};
use crate::pipeline_management::{Pipeline, TextureContainer};

pub struct FrameCache {
    textures: Arena<TextureContainer>,
    mutable_textures: HashMap<(u32,u32),VecDeque<generational_arena::Index>>,
    leased_mutable_textures: HashSet<generational_arena::Index>
}
pub trait WGPUInterface {
    fn get_device(&self) -> wgpu::Device;
    fn get_queue(&self) -> wgpu::Queue;
    fn get_output_format(&self) -> wgpu::TextureFormat;
    fn get_pipeline(&self) -> &Pipeline;
    fn get_output_size(&self) -> (u32,u32);
    fn get_output_texture(&self) -> wgpu::TextureView;

    fn start_encoding(&mut self);
    fn get_encoder(&self) -> Option<&wgpu::CommandEncoder>;
    fn finish_encoding(&mut self);
}

impl FrameCache {
    pub fn create() -> Self {
        return Self {
            textures: Default::default(),
            mutable_textures: Default::default(),
            leased_mutable_textures: Default::default()
        }
    }

    /* cache_size: The size of the texture cache sizes. cache_size_instances: The number of texture objects per cache size. */
    pub fn create_with_buffer_frames(cache_sizes: &[(u32,u32)],cache_size_instances: usize,wgpu_interface: &impl WGPUInterface) -> Self {

        let capacity = cache_sizes.len();

        let mut textures = Arena::with_capacity(capacity);
        let mut mutable_textures = HashMap::with_capacity(capacity);

        for size in cache_sizes.iter() {
            let mut queue = VecDeque::with_capacity(cache_size_instances);

            for _ in 0..cache_size_instances {
                let mutable_texture = TextureContainer::create_mutable(*size,wgpu_interface);
                let index = textures.insert(mutable_texture);

                queue.push_back(index);
            }

            mutable_textures.insert(*size,queue);
        }

        return Self {
            textures,
            mutable_textures,
            leased_mutable_textures: HashSet::default()
        }
    }

    pub fn get_output_frame(&self,wgpu_interface: &impl WGPUInterface) -> Frame {
        return FrameInternal::create_output(wgpu_interface);
    }

    /* Non - statics do not reuse the underlying mutable_textures pool. It is safe to use them across display frames. */
    pub fn create_frame_static(&self,size: (u32,u32),readonly_after_render: bool) -> Frame {
        return match readonly_after_render {
            true => FrameInternal::create_immutable(size,true),
            false => FrameInternal::create_mutable(size),
        }
    }
}

impl FrameCache {
    fn request_mutable_texture(size: (u32,u32)) -> Option<TextureContainer> {
        //todo
        return None;
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
