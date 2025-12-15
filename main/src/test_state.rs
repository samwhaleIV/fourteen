use crate::graphics::Graphics;
use crate::app_state::*;
use wimpy::pipeline_management::Pipeline;

pub struct TestState {
}

pub fn generate_test_state(_graphics: &Graphics,_pipeline: &mut Pipeline) -> AppState {    
    return Box::new(TestState {
    });
}

#[allow(unused_variables)]
impl AppStateHandler for TestState {
    fn unload(&mut self,graphics: &Graphics,pipeline: &mut Pipeline) {
        todo!()
    }

    fn update(&mut self) -> UpdateResult {
        todo!()
    }

    fn render(&self,graphics: &Graphics,pipeline: &mut Pipeline) {
        todo!()
    }

    fn input(&mut self,event: InputEvent) {
        todo!()
    }
}
