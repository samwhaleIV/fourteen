use wgpu::{TextureView};

use crate::app::{AppState, InputEvent, UpdateResult};
use crate::graphics::{Graphics,Vertex};
use crate::app::AppStateHandler;
use crate::paintbrush::{PaintBrush, BufferReference, create_paint_brush};

pub struct TestState {
    paint_brush: PaintBrush,
    vertex_buffer: BufferReference,
    index_buffer: BufferReference
}

pub fn generate_test_state(graphics: &Graphics) -> AppState {    
    let mut paint_brush = create_paint_brush();

    let vertex_buffer = paint_brush.create_vertex_buffer(graphics,&[
        Vertex { position: [-0.0868241, 0.49240386, 0.0], color: [0.5, 0.0, 0.5] }, // A
        Vertex { position: [-0.49513406, 0.06958647, 0.0], color: [0.5, 0.0, 0.5] }, // B
        Vertex { position: [-0.21918549, -0.44939706, 0.0], color: [0.5, 0.0, 0.5] }, // C
        Vertex { position: [0.35966998, -0.3473291, 0.0], color: [0.5, 0.0, 0.5] }, // D
        Vertex { position: [0.44147372, 0.2347359, 0.0], color: [0.5, 0.0, 0.5] }, // E
    ]);

    let index_buffer = paint_brush.create_index_buffer(graphics,&[
        0, 1, 4,
        1, 2, 4,
        2, 3, 4
    ]);

    return Box::new(TestState {
        vertex_buffer,
        paint_brush,
        index_buffer
    });
}

impl AppStateHandler for TestState {
    fn unload(&mut self,_graphics: &Graphics) {
        self.paint_brush.unload();
    }
    fn update(&mut self) -> UpdateResult {
        return UpdateResult::default();
    }

    fn render(&mut self,graphics: &Graphics,render_target: &TextureView) {
        let pb = &mut self.paint_brush;     

        pb.set_clear_color(wgpu::Color::BLACK);

        pb.set_vertex_buffer(&self.vertex_buffer,0,0..self.vertex_buffer.size());
        pb.set_index_buffer(&self.index_buffer,0..self.index_buffer.size());
        pb.draw_indexed_primitives(0..self.index_buffer.len(),0,0..1);

        pb.render(graphics,render_target);
    }
    
    fn input(&mut self,_event: InputEvent) {
        //todo!()
    }
}
