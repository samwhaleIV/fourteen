use std::f32::consts::PI;

use crate::graphics::Graphics;
use crate::app_state::*;

use wimpy::area::Area;
use wimpy::color::Color;
use wimpy::frame::{DrawData, FilterMode, Frame, WrapMode};
use wimpy::pipeline_management::Pipeline;

pub struct TestState {
    texture: Frame
}

pub fn generate_test_state(graphics: &Graphics,pipeline: &mut Pipeline) -> AppState {
    let texture = pipeline.load_texture_bytes(graphics,include_bytes!("../../content/images/null.png"));
    return Box::new(TestState { texture });
}

#[allow(unused_variables)]
impl AppStateHandler for TestState {
    fn unload(&mut self,graphics: &Graphics,pipeline: &mut Pipeline) {
        
    }

    fn update(&mut self) -> UpdateResult {
        return UpdateResult::default();
    }

    fn render(&self,graphics: &Graphics,pipeline: &mut Pipeline) {
        if let Some(f) = &mut pipeline.start(graphics) {
            f.set_texture_filter(FilterMode::Nearest);
            f.set_texture_wrap(WrapMode::Clamp);

            let size = 4;

            let width = (f.width() as f32 / size as f32).ceil() as u32;
            let height = (f.height() as f32 / size as f32).ceil() as u32;

            let quads = width * height;

            let mut buffer = Vec::<DrawData>::with_capacity(quads as usize);

            for x in 0..width {
                for y in 0..height {

                    let x_normal = x as f32 / width as f32;
                    let y_normal = y as f32 / height as f32;

                    buffer.push(DrawData {
                        area: Area {
                            x: (x * size) as f32,
                            y: (y * size) as f32,
                            width: size as f32,
                            height: size as f32
                        },
                        uv: Area::one(),
                        rotation: (x_normal + y_normal) * std::f32::consts::FRAC_PI_8,
                        color: Color {
                            r: (x_normal * 255.0) as u8,
                            g: (y_normal * 255.0) as u8,
                            b: 255,
                            a: u8::MAX
                        },
                    });
                }
            }

            f.draw_frame_set(&self.texture,buffer);

            f.finish(graphics,pipeline);
            pipeline.finish(graphics);
        }
    }

    fn input(&mut self,event: InputEvent) {

    }
}
