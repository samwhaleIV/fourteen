use wgpu::{TextureView};
use collections::named_cache::CacheItemReference;
use crate::camera::Camera;
use crate::graphics::{Graphics, Vertex};
use crate::paintbrush::{PaintBrush, BufferReference};
use crate::app_state::*;

pub struct TestState {
    paint_brush: PaintBrush,
    vertex_buffer: BufferReference,
    index_buffer: BufferReference,
    test_texture: CacheItemReference,
    camera: Camera
}

pub fn generate_test_state(graphics: &mut Graphics) -> AppState {    
    let mut paint_brush = PaintBrush::default();

    let vertex_buffer = paint_brush.create_vertex_buffer(graphics,&[
        Vertex { position: [-0.0868241, 0.49240386], color: [0.0,0.0,0.0], uv: [0.4131759, 0.99240386], }, // A
        Vertex { position: [-0.49513406, 0.06958647], color: [1.0,1.0,1.0], uv: [0.0048659444, 0.56958647], }, // B
        Vertex { position: [-0.21918549, -0.44939706], color: [1.0,1.0,1.0], uv: [0.28081453, 0.05060294], }, // C
        Vertex { position: [0.35966998, -0.3473291], color: [1.0,1.0,1.0], uv: [0.85967, 0.1526709], }, // D
        Vertex { position: [0.44147372, 0.2347359], color: [1.0,1.0,1.0], uv: [0.9414737, 0.7347359], }, // E
    ]);

    let index_buffer = paint_brush.create_index_buffer(graphics,&[
        0, 1, 4,
        1, 2, 4,
        2, 3, 4
    ]);

    let test_texture = graphics.get_texture("Test Texture");

    let camera = Camera::default();

    return Box::new(TestState {
        camera,
        vertex_buffer,
        paint_brush,
        index_buffer,
        test_texture
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

        self.camera.set_aspect_ratio(render_target);
        pb.set_view_projection(self.camera.get_view_projection());

        pb.set_clear_color(wgpu::Color::RED);

        pb.set_texture(self.test_texture);
        pb.set_vertex_buffer(&self.vertex_buffer,0,0..self.vertex_buffer.size());
        pb.set_index_buffer(&self.index_buffer,0..self.index_buffer.size());
        pb.draw_indexed_primitives(0..self.index_buffer.len(),0,0..1);

        pb.render(graphics,render_target);
    }
    
    fn input(&mut self,_event: InputEvent) {
        //todo!()
    }
}
