use std::collections::{HashMap, VecDeque};

use generational_arena::{Arena, Index};
use image::{DynamicImage, ImageError, ImageReader};

use crate::frame::{FrameInternal, Frame};
use crate::lease_arena::LeaseArena;
use crate::texture_container::TextureContainer;
use crate::wgpu_interface::WGPUInterface;

#[derive(Default)]
pub struct FrameCache {
    frames: LeaseArena<(u32,u32),TextureContainer>
}

impl FrameCache {
    pub fn create() -> Self {
        return Self {
            frames: LeaseArena::default()
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

        let frames = LeaseArena::create_with_values(textures,mutable_textures);

        return Self { frames };
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
    pub fn get_mutable_texture_lease(&mut self,size: (u32,u32),wgpu_interface: &impl WGPUInterface) -> Index {
        return self.frames.start_lease(size,||TextureContainer::create_mutable(size,wgpu_interface));
    }
  
    pub fn return_mutable_texture_lease(&mut self,lease: Index) {
        self.frames.end_lease(lease);
    }

    pub fn get_texture(&self,reference: Index) -> &TextureContainer {
        return self.frames.get(reference);
    }

    fn create_finished_frame(&mut self,image: &DynamicImage,wgpu_interface: &impl WGPUInterface) -> Frame {
        let texture_container = TextureContainer::from_image(&image,wgpu_interface);
        let size = texture_container.size();
        let index = self.frames.insert(size,texture_container);
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
