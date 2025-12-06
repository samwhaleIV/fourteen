use std::{collections::{HashMap, VecDeque}, ops::{Range}};

use wgpu::{Buffer, BufferAddress, RenderPass, TextureView, util::{BufferInitDescriptor, DeviceExt}};
use crate::graphics::{self, Graphics, Vertex};

type RenderInstructionQueue = VecDeque<RenderInstruction>;

pub type VertexBufferID = u32;

pub struct VertexBufferReference {
    id: VertexBufferID,
    length: u32
}

impl VertexBufferReference {
    pub fn len(&self) -> u32 {
        return self.length as u32;
    }
}

pub struct PaintBrush {
    render_pass_mode: RenderPassMode,
    instruction_queue: RenderInstructionQueue,
    vertex_buffers: HashMap<VertexBufferID,Buffer>,
    vertex_buffer_counter: VertexBufferID,
    clear_color: wgpu::Color
}

struct DrawPrimitivesData {
    vertices: Range<u32>,
    instances: Range<u32>
}

struct SetVertexBufferData {
    id: VertexBufferID,
    slot: u32,
    buffer_slice: Range<u32>
}

enum RenderInstruction {
    DrawPrimitives(DrawPrimitivesData),
    SetVertexBuffer(SetVertexBufferData)
}

enum RenderPassMode {
    Basic,
    Custom
}

pub fn create_paint_brush() -> PaintBrush {
    return PaintBrush {
        render_pass_mode: RenderPassMode::Basic,
        vertex_buffers: HashMap::new(),
        instruction_queue: VecDeque::new(),
        vertex_buffer_counter: 0,
        clear_color: wgpu::Color::WHITE
    }
}

impl PaintBrush {

    pub fn create_vertex_buffer(&mut self,graphics: &Graphics,vertices: &[Vertex]) -> VertexBufferReference {

        let mut vertex_count = vertices.len();

        if vertex_count > u32::MAX as usize {
            log::error!("Vertex buffer is larger than 'u32' limit. Paint brush cannot address beyond this point.");
            vertex_count = u32::MAX as usize;
        }

        let vertex_buffer = graphics.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let id = self.vertex_buffer_counter;
        self.vertex_buffers.insert(id,vertex_buffer);
        self.vertex_buffer_counter += 1;

        return VertexBufferReference { id, length: vertex_count as u32 };
    }

    pub fn set_clear_color(&mut self,color: wgpu::Color) {
        self.clear_color = color;
    }

    pub fn destroy_vertex_buffer(&mut self,vertex_buffer_ref: &VertexBufferReference) {
        if let Some(vertex_buffer) = self.vertex_buffers.remove(&vertex_buffer_ref.id) {
            vertex_buffer.destroy();
        } else {
            log::error!("Cannot destroy buffer. Vertex buffer with ID '{}' not found.",vertex_buffer_ref.id);
        }
    }

    pub fn unload(&self) {
        for vertex_buffer in self.vertex_buffers.values() {
            vertex_buffer.destroy();
        }
    }

    pub fn draw_primitives(&mut self,vertices: Range<u32>,instances: Range<u32>) {
        self.instruction_queue.push_back(RenderInstruction::DrawPrimitives(DrawPrimitivesData {
            vertices,instances
        }));
    }

    pub fn set_vertex_buffer(&mut self,vertex_buffer_ref: &VertexBufferReference,slot: u32,buffer_slice: Range<u32>) {
        self.instruction_queue.push_back(RenderInstruction::SetVertexBuffer(SetVertexBufferData {
            id: vertex_buffer_ref.id, slot, buffer_slice
        }));
    }
  
    pub fn render(&mut self,graphics: &Graphics,render_target: &TextureView) {

        if self.instruction_queue.is_empty() {
            log::warn!("Can't render because rendering instruction queue is empty.");
            return;
        }

        let mut encoder = graphics.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder")
        });

        {
            let mut render_pass = match self.render_pass_mode {
                RenderPassMode::Basic => graphics::get_basic_render_pass(&mut encoder,&render_target,self.clear_color),
                RenderPassMode::Custom => todo!("Not implemented.")
            };

            graphics.set_pipeline(&mut render_pass,graphics::PipelineVariant::Basic);

            for instruction in self.instruction_queue.iter() {
                self.execute_instruction(&mut render_pass,instruction);
            }
            self.instruction_queue.clear();
        }

        graphics.queue.submit(std::iter::once(encoder.finish()));
    }

    fn execute_instruction(&self,render_pass: &mut RenderPass,instruction: &RenderInstruction) {
        match instruction {
            RenderInstruction::DrawPrimitives(data) => {
                render_pass.draw(data.vertices.clone(),data.instances.clone());
            },
            RenderInstruction::SetVertexBuffer(data) => {
                if let Some(vertex_buffer) = self.vertex_buffers.get(&data.id) {

                    let start = (data.buffer_slice.start * Vertex::SIZE) as BufferAddress;
                    let end = (data.buffer_slice.end * Vertex::SIZE) as BufferAddress;

                    render_pass.set_vertex_buffer(data.slot,vertex_buffer.slice(start..end));
                } else {
                    panic!("Vertex buffer with ID '{}' not found.",data.id)
                }
            }
            _ => {}
        }
    }
}
