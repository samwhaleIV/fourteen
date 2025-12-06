use wgpu::{TextureView};

use crate::app::{AppState, InputEvent, UpdateResult};
use crate::graphics::{Graphics,Vertex};
use crate::app::AppStateHandler;
use crate::paintbrush::{PaintBrush, VertexBufferReference, create_paint_brush};

pub struct TestState {
    paint_brush: PaintBrush,
    vertex_buffer: VertexBufferReference
}

pub fn generate_test_state(graphics: &Graphics) -> AppState {    
    let mut paint_brush = create_paint_brush();

    let vertex_buffer = paint_brush.create_vertex_buffer(graphics,&[
        Vertex { position: [0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0] },
        Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] },
        Vertex { position: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] },
    ]);

    return Box::new(TestState {vertex_buffer,paint_brush});
}

impl AppStateHandler for TestState {
    fn unload(&mut self,_graphics: &Graphics) {
        self.paint_brush.unload();
    }
    fn update(&mut self) -> UpdateResult {
        return UpdateResult::default();
    }

    fn render(&mut self,graphics: &Graphics,render_target: &TextureView) {

        let size = self.vertex_buffer.len();

        self.paint_brush.set_clear_color(wgpu::Color::BLACK);

        self.paint_brush.set_vertex_buffer(&self.vertex_buffer,0,0..size);
        self.paint_brush.draw_primitives(0..size,0..1);

        self.paint_brush.render(graphics,render_target);
    }
    
    fn input(&mut self,_event: InputEvent) {
        //todo!()
    }
}
