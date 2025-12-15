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

            f.draw_frame(&self.texture,DrawData {
                area: Area {
                    x: 0.0,
                    y: 0.0,
                    width: 0.5,
                    height: 0.5
                },
                uv: Area::one(),
                rotation: 0.0,
                color: Color::WHITE,
            });

            f.finish(graphics,pipeline);
            pipeline.finish(graphics);
        }
    }

    fn input(&mut self,event: InputEvent) {

    }
}
