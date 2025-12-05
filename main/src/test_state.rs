use wgpu::RenderPass;

use crate::app::UpdateResult;
use crate::graphics::Graphics;
use crate::app::AppStateHandler;
use crate::app::AppOperation;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

const VERTICES: &[Vertex] = &[
    Vertex { position: [0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0] },
    Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] },
    Vertex { position: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] },
];

#[derive(Default)]
pub struct TestState;

impl AppStateHandler for TestState {
    fn load(&mut self,_graphics: &Graphics) {

    }

    fn unload(&mut self,_graphics: &Graphics) {

    }
    fn update(&mut self) -> UpdateResult {
        return UpdateResult::default();
    }

    fn render(&mut self,render_pass: &mut RenderPass) {
        render_pass.draw(0..3, 0..1); // 3.
    }
}
