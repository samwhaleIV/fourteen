use wgpu::{Buffer, RenderPass, RenderPipeline, TextureView};
use wgpu::util::{DeviceExt,BufferInitDescriptor};

use crate::app::{AppState, InputEvent, UpdateResult};
use crate::graphics::{self, Graphics};
use crate::app::AppStateHandler;
use crate::app::AppOperation;
use crate::paintbrush::{PaintBrush, create_paint_brush};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}
 

const VERTICES: &[Vertex] = &[
    Vertex { position: [0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0] },
    Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] },
    Vertex { position: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] },
];

pub struct TestState {
    paint_brush: PaintBrush,
    vertex_buffer: Buffer
}

pub fn generate_test_state(graphics: &Graphics) -> AppState {
    let vertex_buffer = graphics.device.create_buffer_init(&BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(VERTICES),
        usage: wgpu::BufferUsages::VERTEX,
    }); 
    return Box::new(TestState {
        vertex_buffer,
        paint_brush: create_paint_brush()
    });
}

impl AppStateHandler for TestState {
    fn unload(&mut self,_graphics: &Graphics) {

    }
    fn update(&mut self) -> UpdateResult {
        return UpdateResult::default();
    }

    fn render(&mut self,graphics: &Graphics,render_target: &TextureView) {
        self.paint_brush.draw_primitives(0..3,0..1);
        
        self.paint_brush.render(graphics,render_target);
    }
    
    fn input(&mut self,event: InputEvent) {
        //todo!()
    }
}
