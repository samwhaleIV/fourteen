#![allow(unused_variables)]

use wimpy::app::*;
use wimpy::wgpu::GraphicsContext;

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

    fn render(&mut self,context: &mut AppContext<SharedState>) {
        todo!()
    }

    fn update(&mut self,context: &mut AppContext<SharedState>) -> UpdateResult<SharedState> {
        todo!()
    }
}
