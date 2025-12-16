//#![allow(dead_code,unused_variables)]

use image::ImageError;

use crate::{
    area::{Area, Layout, LayoutDimension, Position, Size}, color::Color, frame::DrawData, pipeline_management::{CacheOptions, FrameConfig, FrameLifetime, Pipeline, PipelineCreationOptions}, wgpu_interface::WGPUInterface
};


mod lease_arena;
mod texture_container;
mod frame_processor;

pub mod frame;
pub mod color;
pub mod area;

pub mod pipeline_management;
pub mod wgpu_interface;

struct VirtualWGPUProvider {

}

impl WGPUInterface for VirtualWGPUProvider {
    fn get_device(&self) -> &wgpu::Device {
        todo!()
    }

    fn get_queue(&self) -> &wgpu::Queue {
        todo!()
    }

    fn get_output_format(&self) -> wgpu::TextureFormat {
        todo!()
    }
    
    fn get_output_surface(&self) -> Option<wgpu::SurfaceTexture> {
        todo!()
    }
}

const MAX_QUADS: u32 = 1000;
const MAX_UNIFORMS: u32 = 100;

#[allow(dead_code)]
fn test() -> Result<(),ImageError> {

    let mut w = VirtualWGPUProvider {
    /*
        This trait is where the magic binding happens (device, queue, swapchain output).
        This way, the 2D renderer is windowing system agnostic.
    */
    };

    let mut pipeline = Pipeline::create(&w,PipelineCreationOptions {
        quad_instance_capacity: MAX_QUADS,
        uniform_capacity: MAX_UNIFORMS,
        cache_options: Some(CacheOptions { instances: 4, sizes: vec![(64,64)] })
    });

    let texture_frame = pipeline.load_texture(&w,"hello_world.png")?;

    /* output_frame obtained from swapchain */
    if let Some(output_frame) = &mut pipeline.start(&mut w) {

        /* Each frame has its own filter mode state. */
        output_frame.set_texture_filter(frame::FilterMode::Nearest);
        output_frame.set_texture_wrap(frame::WrapMode::Clamp);

        /*
            Doesn't create a new texture view, just reuses one we defined in the cache_options.
            If a frame backing does not exist of this size, one will be created on demand.
        */
        let mut intermediate_frame = pipeline.get_frame(&w,FrameConfig {
            /* Lives until pipeline.finish() */
            lifetime: FrameLifetime::Temporary,
            size: (64,64),
            draw_once: true,
        });

        intermediate_frame.draw_frame(&texture_frame,DrawData {
            area: intermediate_frame.area(),
            color: Color::RED, //Tint the texture red
            ..Default::default()
        });

        /*
            A system for computing render destinations with basic layout rules.
            Can help position frames relative to other frames.
        */

        let output_area = Layout::same_xy(LayoutDimension {
            position: Position::center_of_parent(),
            size: Size::of_parent_height(0.1),
            ..Default::default()
        }).compute(output_frame.area());

        output_frame.draw_frame(&intermediate_frame,DrawData {
            area: output_area,
            color: Color::WHITE,
            ..Default::default()
        });

        /*
            'Finish' calls can be placed anywhere, however,
            they have to be executed in render pass order;

            Each 'finish()' is conceptually a render pass.
            If frame 'B' draws into frame 'A', then B.finish() must happen before A.finish()
        */

        intermediate_frame.finish(&w,&mut pipeline);
        output_frame.finish(&w,&mut pipeline);

        pipeline.finish(&mut w);
    }

    return Ok(());
}
