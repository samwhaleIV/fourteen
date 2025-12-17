#![allow(unused_variables)]

use wimpy::app::*;
use wimpy::graphics::GraphicsContext;

pub struct TestState { }

pub fn generate_test_state(device: &VirtualDevice,context: &mut GraphicsContext) -> AppState {
    return Box::new(TestState {});
}

impl AppStateInterface for TestState {
    fn unload(&mut self,device: &VirtualDevice,context: &mut GraphicsContext) {
        todo!()
    }

    fn update(&mut self) -> UpdateResult {
        todo!()
    }

    fn input(&mut self,event: InputEvent) {
        todo!()
    }

    fn render(&self,device: &VirtualDevice,context: &mut GraphicsContext) {

        if let Some(mut output_frame) = context.start(device) {

            


            output_frame.finish(device,context);

            context.finish(device);
        }
    }
}
