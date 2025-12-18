#![allow(unused_variables)]

use wimpy::app::*;
use wimpy::graphics::GraphicsContext;

use crate::shared_state::SharedState;

pub struct SharedState {
    pub ahhhhh: u32
}

impl SharedState {
    pub fn generator(_device: &VirtualDevice,_context: &mut GraphicsContext) -> Self {
        return Self {
            ahhhhh: 0
        }
    }
}


pub struct TestState {
    shared_state: Option<SharedState>
}

pub fn generate_test_state(device: &VirtualDevice,context: &mut GraphicsContext) -> AppState<SharedState> {
    return Box::new(TestState {
        shared_state: None
    });
}

impl AppStateInterface<SharedState> for TestState {
    fn input(&mut self,input_event: InputEvent) {
        todo!()
    }

    fn unload(&mut self,virtual_device: &VirtualDevice,graphics_context: &mut GraphicsContext) {
        todo!()
    }

    fn render(&mut self,virtual_device: &VirtualDevice,graphics_context: &mut GraphicsContext) {
        todo!()
    }

    fn update(&mut self,virtual_device: &VirtualDevice,graphics_context: &mut GraphicsContext) -> UpdateResult<SharedState> {
        todo!()
    }

    fn insert_shared_state(&mut self,shared_state: Option<SharedState>) {
        self.shared_state = shared_state;
    }

    fn remove_shared_state(&mut self) -> Option<SharedState> {
        return self.shared_state.take();
    }
}
