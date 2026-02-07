use std::{
    num::NonZero,
    ops::Range
};

use gltf::{
    Primitive,
    buffer::Data
};

use wgpu::{
    Buffer,
    BufferDescriptor,
    Device,
    Queue,
};

use crate::wgpu::pipelines::ModelVertex;

pub struct ModelCache {
    index_buffer: Buffer,
    vertex_buffer: Buffer,

    index_buffer_length: usize,
    vertex_buffer_length: usize,

    needs_to_flush: bool
}

pub struct ModelCacheEntry {
    pub index_range: Range<u32>,
    pub base_vertex: i32,
}

pub enum ModelImportError {
    MissingIndices,
    MissingPositions,
    MismatchedAttributeQuantity,
    EmptyVertexBuffer,
    EmptyIndexBuffer,
    VertexBufferWriteFailure,
    IndexBufferWriteFailure
}

const DIFFUSE_UV_CHANNEL: u32 = 0;
const LIGHTMAP_UV_CHANNEL: u32 = 1;

impl ModelCache {
    pub fn create(device: &Device,vertex_buffer_size: usize,index_buffer_size: usize) -> Self {
        
        let index_buffer = device.create_buffer(&BufferDescriptor{
            label: Some("Model Cache Index Buffer"),      
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            size: index_buffer_size as u64,
            mapped_at_creation: false,
        });

        let vertex_buffer = device.create_buffer(&BufferDescriptor{
            label: Some("Model Cache Vertex Buffer"),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            size: vertex_buffer_size as u64,
            mapped_at_creation: false,
        });

        return Self {
            index_buffer,
            vertex_buffer,
            index_buffer_length: 0,
            vertex_buffer_length: 0,
            needs_to_flush: false,
        }
    }

    pub fn create_entry(&mut self,queue: &Queue,buffers: Vec<Data>,mesh: Primitive) -> Result<ModelCacheEntry,ModelImportError> {
        let reader = mesh.reader(|buffer|Some(&buffers[buffer.index()]));

        let positions: Vec<[f32;3]> = match reader.read_positions() {
            Some(value) => value.collect(),
            None => return Err(ModelImportError::MissingPositions),
        };

        let indices: Vec<u32> = match reader.read_indices() {
            Some(value) => value.into_u32().collect(),
            None => return Err(ModelImportError::MissingIndices),
        };

        let diffuse_uvs: Vec<[f32;2]> = match reader.read_tex_coords(DIFFUSE_UV_CHANNEL) {
            Some(value) => value.into_f32().collect(),
            None => vec![[0.0;2];positions.len()],
        };

        let lightmap_uvs: Vec<[f32;2]> = match reader.read_tex_coords(LIGHTMAP_UV_CHANNEL) {
            Some(value) => value.into_f32().collect(),
            None => vec![[0.0;2];positions.len()],
        };

        if
            diffuse_uvs.len() != positions.len() ||
            lightmap_uvs.len() != positions.len()
        {
            return Err(ModelImportError::MismatchedAttributeQuantity);
        }

        let vertex_buffer_stride = positions.len() * size_of::<ModelVertex>();
        let index_buffer_stride = indices.len() * size_of::<u32>();

        let Some(mut vertex_buffer_view) = queue.write_buffer_with(
            &self.vertex_buffer,
            (self.vertex_buffer_length * size_of::<ModelVertex>()) as u64,
            match NonZero::new(vertex_buffer_stride as u64) {
                Some(value) => value,
                None => return Err(ModelImportError::EmptyVertexBuffer),
            }
        ) else {
            return Err(ModelImportError::VertexBufferWriteFailure);
        };

        let Some(mut index_buffer_view) = queue.write_buffer_with(
            &self.index_buffer,
            (self.index_buffer_length * size_of::<u32>()) as u64,
            match NonZero::new(index_buffer_stride as u64) {
                Some(value) => value,
                None => return Err(ModelImportError::EmptyIndexBuffer),
            }
        ) else {
            return Err(ModelImportError::IndexBufferWriteFailure);
        };

        let mut vertices: Vec<ModelVertex> = Vec::with_capacity(positions.len());

        for i in 0..positions.len() {
            let vertex = ModelVertex {
                diffuse_uv: diffuse_uvs[i],
                lightmap_uv:lightmap_uvs[i],
                position: positions[i],
                _padding: Default::default()
            };
            vertices[i] = vertex;
        }

        vertex_buffer_view.copy_from_slice(bytemuck::cast_slice(&vertices));
        index_buffer_view.copy_from_slice(bytemuck::cast_slice(&indices));

        let entry = ModelCacheEntry {
            index_range: Range {
                start: self.index_buffer_length  as u32,
                end: (self.index_buffer_length + index_buffer_stride) as u32
            },
            base_vertex: self.vertex_buffer_length as i32,
        };

        self.vertex_buffer_length += vertex_buffer_stride;
        self.index_buffer_length += index_buffer_stride;

        self.needs_to_flush = true;

        return Ok(entry);
    }

    pub fn needs_to_flush(&self) -> bool {
        self.needs_to_flush
    }
}
