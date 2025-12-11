#![allow(dead_code)]

use std::{
    collections::{HashMap,VecDeque},
    ops::{Range}
};

use wgpu::{
    Buffer,
    BufferAddress,
    RenderPass,
    TextureView,
    util::BufferInitDescriptor,
};

use crate::{graphics::{
    self,
    Graphics,
    Vertex,
    ViewProjection,
}};

use collections::named_cache::CacheItemReference;

#[derive(Default)]
pub struct PaintBrush {
    instruction_queue: VecDeque<RenderInstruction>,
    buffer_map: HashMap<usize,Buffer>,
    id_counter: usize,
    clear_color: wgpu::Color
}

#[derive(Clone,Copy)]
pub struct BufferReference {
    id: usize,
    size: u32,
    length: u32,
    buffer_type: BufferType
}

impl BufferReference {
    pub fn len(&self) -> u32 {
        return self.length;
    }
    pub fn size(&self) -> u32 {
        return self.size;
    }
}

struct DrawPrimitivesData {
    vertices: Range<u32>,
    instances: Range<u32>
}

struct SetVertexBufferData {
    id: usize,
    slot: u32,
    buffer_slice: Range<u32>
}

struct SetIndexBufferData {
    id: usize,
    buffer_slice: Range<u32>
}

struct DrawIndexedPrimitivesData {
    indices: Range<u32>,
    base_vertex: i32,
    instances: Range<u32>
}

enum RenderInstruction {
    DrawPrimitives(DrawPrimitivesData),
    SetVertexBuffer(SetVertexBufferData),
    SetIndexBuffer(SetIndexBufferData),
    DrawIndexedPrimitives(DrawIndexedPrimitivesData),
    SetTexture(CacheItemReference),
    SetViewProjection(Box<ViewProjection>)
}

#[derive(Debug,PartialEq,Eq,Clone,Copy)]
enum BufferType {
    Vertex,
    Index
}

impl PaintBrush {

    pub fn unload(&self) {
        for buffer in self.buffer_map.values() {
            buffer.destroy();
        }
    }

    fn create_buffer(&mut self,graphics: &Graphics,buffer_type: BufferType,contents: &[u8],length: u32) -> BufferReference {
        let mut buffer_size = contents.len();

        if buffer_size > u32::MAX as usize {
            log::error!("Buffer is larger than 'u32' limit. Paint brush cannot address beyond this point.");
            buffer_size = u32::MAX as usize;
        }

        let (label,usage) = match buffer_type {
            BufferType::Vertex => ("Vertex Buffer",wgpu::BufferUsages::VERTEX),
            BufferType::Index => ("Index Buffer",wgpu::BufferUsages::INDEX)
        };

        let id = self.id_counter;
        self.id_counter += 1;

        self.buffer_map.insert(id,graphics.create_buffer(&BufferInitDescriptor{
            label: Some(label),
            contents,
            usage
        }));

        return BufferReference {
            id,
            buffer_type,
            length,
            size: buffer_size as u32,
        };
    }

    pub fn create_vertex_buffer(&mut self,graphics: &Graphics,vertices: &[Vertex]) -> BufferReference {
        return self.create_buffer(
            graphics,
            BufferType::Vertex,
            bytemuck::cast_slice(vertices),
            vertices.len() as u32
        );
    }

    pub fn create_index_buffer(&mut self,graphics: &Graphics,indices: &[u32]) -> BufferReference {
        return self.create_buffer(
            graphics,
            BufferType::Index,
            bytemuck::cast_slice(indices),
            indices.len() as u32
        );
    }

    pub fn set_clear_color(&mut self,color: wgpu::Color) {
        self.clear_color = color;
    }

    pub fn destroy_buffer(&mut self,buffer_ref: &BufferReference) {
        if let Some(buffer) = self.buffer_map.get(&buffer_ref.id) {
            buffer.destroy();
            self.buffer_map.remove(&buffer_ref.id);
        } else {
            log::error!("Cannot destroy buffer. Vertex buffer with ID '{}' not found.",buffer_ref.id);
        }
    }

    pub fn draw_primitives(&mut self,vertices: Range<u32>,instances: Range<u32>) {
        self.instruction_queue.push_back(RenderInstruction::DrawPrimitives(DrawPrimitivesData {
            vertices,instances
        }));
    }

    pub fn draw_indexed_primitives(&mut self,indices: Range<u32>, base_vertex: i32, instances: Range<u32>) {
        self.instruction_queue.push_back(RenderInstruction::DrawIndexedPrimitives(DrawIndexedPrimitivesData { 
            indices, base_vertex, instances
        }));
    }

    pub fn set_vertex_buffer(&mut self,vertex_buffer_ref: &BufferReference,slot: u32,buffer_slice: Range<u32>) {
        assert_eq!(vertex_buffer_ref.buffer_type,BufferType::Vertex,"Invalid buffer reference: This reference is not registered as a vertex buffer.");

        self.instruction_queue.push_back(RenderInstruction::SetVertexBuffer(SetVertexBufferData {
            id: vertex_buffer_ref.id, slot, buffer_slice
        }));
    }

    pub fn set_index_buffer(&mut self,index_buffer_ref: &BufferReference,buffer_slice: Range<u32>) {
        assert_eq!(index_buffer_ref.buffer_type,BufferType::Index,"Invalid buffer reference: This reference is not registered as an index buffer.");

        self.instruction_queue.push_back(RenderInstruction::SetIndexBuffer(SetIndexBufferData {
            id: index_buffer_ref.id, buffer_slice
        }));
    }

    pub fn set_view_projection(&mut self,view_projection: Box<ViewProjection>) {
        self.instruction_queue.push_back(RenderInstruction::SetViewProjection(view_projection));
    }

    pub fn set_texture(&mut self,texture_reference: CacheItemReference) {
        self.instruction_queue.push_back(RenderInstruction::SetTexture(texture_reference));
    }
  
    pub fn render(&mut self,graphics: &Graphics,render_target: &TextureView) {

        if self.instruction_queue.is_empty() {
            log::warn!("Can't render because rendering instruction queue is empty.");
            return;
        }

        let mut encoder = graphics.create_command_encoder();

        let default_pipeline = graphics.get_default_pipeline();

        {
            let mut render_pass: RenderPass = graphics::get_basic_render_pass(&mut encoder,&render_target,self.clear_color);
            render_pass.set_pipeline(default_pipeline);

            for instruction in self.instruction_queue.iter() {
                self.execute_instruction(graphics,&mut render_pass,instruction);
            }
            self.instruction_queue.clear();
        }

        graphics.submit_encoder(encoder);
    }

    fn execute_instruction(&self,graphics: &Graphics,render_pass: &mut RenderPass,instruction: &RenderInstruction) {
        match instruction {
            RenderInstruction::DrawPrimitives(data) => {
                render_pass.draw(data.vertices.clone(),data.instances.clone());
            },
            RenderInstruction::DrawIndexedPrimitives(data) => {
                render_pass.draw_indexed(data.indices.clone(),data.base_vertex,data.instances.clone());
            },
            RenderInstruction::SetVertexBuffer(data) => {
                if let Some(vertex_buffer) = self.buffer_map.get(&data.id) {

                    let start = data.buffer_slice.start as BufferAddress;
                    let end = data.buffer_slice.end as BufferAddress;

                    render_pass.set_vertex_buffer(data.slot,vertex_buffer.slice(start..end));
                } else {
                    panic!("Vertex buffer with ID '{}' not found.",data.id)
                }
            },
            RenderInstruction::SetIndexBuffer(data) => {
                if let Some(index_buffer) = self.buffer_map.get(&data.id) {

                    let start = data.buffer_slice.start as BufferAddress;
                    let end = data.buffer_slice.end as BufferAddress;

                    render_pass.set_index_buffer(index_buffer.slice(start..end),wgpu::IndexFormat::Uint32);
                } else {
                    panic!("Vertex buffer with ID '{}' not found.",data.id)
                }
            },
            RenderInstruction::SetTexture(texture_reference) => {
                let bind_group = graphics.get_bind_group(texture_reference);
                render_pass.set_bind_group(graphics::TEXTURE_BIND_GROUP_INDEX,bind_group,&[]);
            },
            RenderInstruction::SetViewProjection(view_projection) => {
                let bind_group = graphics.write_view_projection(view_projection);
                render_pass.set_bind_group(graphics::VIEW_PROJECTION_BIND_GROUP_INDEX,bind_group,&[]);
            }
        }
    }
}
