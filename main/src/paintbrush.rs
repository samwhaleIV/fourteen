use std::{
    collections::{HashMap,VecDeque},
    ops::{Range}
};

use wgpu::{
    BindGroup,
    Buffer,
    BufferAddress,
    RenderPass,
    TextureView,
    util::{BufferInitDescriptor,DeviceExt}
};

use crate::graphics::{
    self,
    BindGroupReference,
    BindGroupType,
    Graphics,
    PipelineVariant,
    Vertex,
    ViewProjection,
    ViewProjectionMatrix,
    VIEW_PROJECTION_BIND_GROUP_INDEX,
};

pub struct PaintBrush {
    instruction_queue: RenderInstructionQueue,
    buffers: HashMap<u32,Buffer>,
    buffer_counter: u32,
    clear_color: wgpu::Color,
    view_projection_uniform_handle: ViewProjectionUniformHandle
}

struct ViewProjectionUniformHandle {
    buffer: Buffer,
    bind_group: BindGroup
}

type RenderInstructionQueue = VecDeque<RenderInstruction>;
pub struct BufferReference {
    id: u32,
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
    id: u32,
    slot: u32,
    buffer_slice: Range<u32>
}

struct SetIndexBufferData {
    id: u32,
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
    SetTexture(BindGroupReference),
    SetViewProjection(Box<ViewProjection>)
}

pub fn create_paint_brush(graphics: &Graphics) -> PaintBrush {

    let view_projection_buffer = graphics.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("View Projection Buffer"),
        contents: bytemuck::cast_slice(&ViewProjectionMatrix::default()),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let view_projection_bind_group = graphics.device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &graphics.get_pipeline(PipelineVariant::Basic).get_bind_group_layout(VIEW_PROJECTION_BIND_GROUP_INDEX),
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: view_projection_buffer.as_entire_binding(),
        }],
        label: Some("View Projection Bind Group"),
    });

    return PaintBrush {
        buffers: HashMap::new(),
        instruction_queue: VecDeque::new(),
        buffer_counter: 0,
        clear_color: wgpu::Color::WHITE,
        view_projection_uniform_handle: ViewProjectionUniformHandle {
            buffer: view_projection_buffer,
            bind_group: view_projection_bind_group
        }
    }
}

#[derive(Debug,PartialEq,Eq)]
enum BufferType {
    Vertex,
    Index
}

impl PaintBrush {

    pub fn unload(&self) {
        for buffer in self.buffers.values() {
            buffer.destroy();
        }
        self.view_projection_uniform_handle.buffer.destroy();
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

        let buffer = graphics.device.create_buffer_init(&BufferInitDescriptor {
            label: Some(label),
            contents: contents,
            usage
        });

        let id = self.buffer_counter;
        self.buffers.insert(id,buffer);
        self.buffer_counter += 1;

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
        if let Some(buffer) = self.buffers.remove(&buffer_ref.id) {
            buffer.destroy();
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

    pub fn set_view_projection(&mut self,view_projection: Box<ViewProjection>) {
        self.instruction_queue.push_back(RenderInstruction::SetViewProjection(view_projection));
    }

    pub fn set_index_buffer(&mut self,index_buffer_ref: &BufferReference,buffer_slice: Range<u32>) {
        assert_eq!(index_buffer_ref.buffer_type,BufferType::Index,"Invalid buffer reference: This reference is not registered as an index buffer.");

        self.instruction_queue.push_back(RenderInstruction::SetIndexBuffer(SetIndexBufferData {
            id: index_buffer_ref.id, buffer_slice
        }));
    }

    pub fn set_texture(&mut self,texture_reference: BindGroupReference) {
        assert_eq!(
            texture_reference.bind_group_type,
            BindGroupType::Texture,
            "Invalid bind group reference. Bind group reference is not a texture."
        );
        self.instruction_queue.push_back(RenderInstruction::SetTexture(texture_reference));
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
            let mut render_pass = graphics::get_basic_render_pass(&mut encoder,&render_target,self.clear_color);

            graphics.set_pipeline(&mut render_pass,graphics::PipelineVariant::Basic);

            for instruction in self.instruction_queue.iter() {
                self.execute_instruction(graphics,&mut render_pass,instruction);
            }
            self.instruction_queue.clear();
        }

        graphics.queue.submit(std::iter::once(encoder.finish()));
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
                if let Some(vertex_buffer) = self.buffers.get(&data.id) {

                    let start = data.buffer_slice.start as BufferAddress;
                    let end = data.buffer_slice.end as BufferAddress;

                    render_pass.set_vertex_buffer(data.slot,vertex_buffer.slice(start..end));
                } else {
                    panic!("Vertex buffer with ID '{}' not found.",data.id)
                }
            },
            RenderInstruction::SetIndexBuffer(data) => {
                if let Some(index_buffer) = self.buffers.get(&data.id) {

                    /* Might not need to multiply by index size ? */
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
                let data = view_projection.get_bytes();
                let handle = &self.view_projection_uniform_handle;
                graphics.queue.write_buffer(&handle.buffer,0,data);
                let bind_group = &handle.bind_group;
                render_pass.set_bind_group(VIEW_PROJECTION_BIND_GROUP_INDEX,bind_group,&[]);
            },
            _ => panic!("Render instruction not implemented.")
        }
    }
}
