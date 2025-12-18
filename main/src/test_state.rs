#![allow(unused_variables)]

use wimpy::app::*;
use wimpy::wgpu::{FilterMode, GraphicsContext};

#[allow(unused)]
pub struct SharedState {
    pub ahhhhh: u32
}

impl SharedState {
    pub fn generator(_context: &mut GraphicsContext<VirtualDevice>) -> Self {
        return Self {
            ahhhhh: 0
        }
    }
}

#[allow(unused)]
pub struct TestState {
    shared_state: Option<SharedState>
}

pub fn generate_test_state(context: &mut AppContext<SharedState>) -> AppState<SharedState> {
    return Box::new(TestState {
        shared_state: None
    });
}

impl AppStateInterface<SharedState> for TestState {
    fn input(&mut self,input_event: InputEvent,context: &mut AppContext<SharedState>) {
        todo!()
    }

    fn unload(&mut self,context: &mut AppContext<SharedState>) {
        todo!()
    }

fn render(&mut self,app_context: &mut AppContext<SharedState>) {
    let gfx = app_context.graphics();

    if let Some(mut output_frame) = gfx.create_output_frame() {
        output_frame.set_texture_filter(FilterMode::Linear);
        output_frame.draw( ... );

        gfx.bake(&mut output_frame);
        gfx.present_output_frame();
    }
}

    fn update(&mut self,context: &mut AppContext<SharedState>) -> UpdateResult<SharedState> {
        todo!()
    }
}
