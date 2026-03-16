use crate::app::graphics::WimpyTextureKey;

use super::pipelines::pipeline_3d::MeshVertex;
use std::{marker::PhantomData, num::NonZero};
use bytemuck::{Pod,Zeroable};
use slotmap::SlotMap;
use wgpu::*;

use gltf::{
    Document,
    Mesh,
    Primitive,
    buffer::Data,
};

const TEXTURED_MESH_REFERENCE_START_CAPACITY: usize = 8;

slotmap::new_key_type! {
    pub struct TexturedMeshReference;
}

pub struct MeshCache {
    mesh_references: SlotMap<TexturedMeshReference,Vec<TexturedMeshlet>>,
    vertices: TypedBuffer<MeshVertex>,
    indices: TypedBuffer<u32>
}

#[derive(Debug)]
pub struct TexturedMeshlet {
    pub range: MeshletRange,
    pub diffuse: WimpyTextureKey,
    pub lightmap: WimpyTextureKey,
}

pub struct TypedBuffer<T> {
    value: wgpu::Buffer,
    // How many instances are in the buffer
    logical_length: usize,

    /// The offset for the next write to the buffer
    physical_length: BufferAddress,
    phantom: PhantomData<T>,
}

struct BufferWriteFrame {
    view: QueueWriteBufferView,
    stride: BufferAddress
}

impl<T> TypedBuffer<T>
where
    T: Pod + Zeroable
{
    fn new(buffer: Buffer) -> Self {
        return TypedBuffer {
            value: buffer,
            logical_length: 0,
            physical_length: 0,
            phantom: Default::default()
        }
    }

    fn get_view(&self,queue: &Queue,length: usize) -> Option<BufferWriteFrame> {
        let stride = (length * size_of::<T>()) as BufferAddress;
        match queue.write_buffer_with(
            &self.value,
            self.physical_length,
            match NonZero::new(stride) {
                Some(value) => value,
                None => return None
            }
        ) {
            Some(view) => Some(BufferWriteFrame {
                view,
                stride,
            }),
            None => None,
        }
    }

    fn write(&mut self,queue: &Queue,values: &[T]) -> bool {
        let num_of_instances = values.len();
        let Some(mut frame) = self.get_view(queue,num_of_instances) else {
            return false;
        };

        frame.view.copy_from_slice(bytemuck::cast_slice(&values));

        self.logical_length += num_of_instances;
        self.physical_length += frame.stride;

        return true;
    }

    pub fn get_buffer(&self) -> &Buffer {
        return &self.value;
    }
}

#[derive(Debug)]
pub enum ModelError {
    GltfParseFailure(String),

    NoMeshes,
    NoRenderPrimitive,

    MissingIndices,
    MissingPositions,

    MissingUVs,
    MismatchedAttributeQuantity,

    EmptyVertexBuffer,
    EmptyIndexBuffer,

    VertexBufferWriteFailure,
    IndexBufferWriteFailure,

    TriMeshCreationFailure(String)
}

const DIFFUSE_UV_CHANNEL: u32 = 0;
const LIGHTMAP_UV_CHANNEL: u32 = 1;

#[derive(Debug)]
pub struct MeshletRange {
    pub index_start: u32,
    pub index_count: u32,
    pub base_vertex: u32,
}

impl MeshCache {
    fn import_render_primitive(
        &mut self,
        buffers: &Vec<Data>,
        queue: &Queue,
        primitive: Primitive,
    ) -> Result<MeshletRange,ModelError> {
        let reader = primitive.reader(|buffer|Some(&buffers[buffer.index()]));

        let positions: Vec<[f32;3]> = match reader.read_positions() {
            Some(value) => value.collect(),
            None => return Err(ModelError::MissingPositions),
        };

        let indices: Vec<u32> = match reader.read_indices() {
            Some(value) => value.into_u32().collect(),
            None => return Err(ModelError::MissingIndices),
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
            return Err(ModelError::MismatchedAttributeQuantity);
        }

        let mut vertices: Vec<MeshVertex> = Vec::with_capacity(positions.len());

        for i in 0..positions.len() {
            let vertex = MeshVertex {
                uv_diffuse: diffuse_uvs[i],
                uv_lightmap:lightmap_uvs[i],
                position: positions[i],
                _padding: 0.0
            };
            vertices.push(vertex);
        }

        let base_vertex = self.vertices.logical_length;

        if !self.vertices.write(queue,&vertices) {
            return Err(ModelError::VertexBufferWriteFailure);
        };

        let index_start = self.indices.logical_length;
        if !self.indices.write(queue,&indices) {
            return Err(ModelError::IndexBufferWriteFailure);
        };

        let index_end = self.indices.logical_length;

        let entry = MeshletRange {
            base_vertex: base_vertex as u32,
            index_start: index_start as u32,
            index_count: (index_end - index_start) as u32,
        };

        return Ok(entry);
    }
}

fn find_model_mesh<'a>(document: &'a Document) -> Option<Mesh<'a>> {
    for mesh in document.meshes() {
        return Some(mesh)
    }
    return None;
}

impl MeshCache {
    pub fn create(device: &Device,vertex_buffer_size: usize,index_buffer_size: usize) -> Self {
        let indices = TypedBuffer::new(device.create_buffer(&BufferDescriptor{
            label: Some("Model Cache Index Buffer"),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            size: index_buffer_size as BufferAddress,
            mapped_at_creation: false,
        }));

        let vertices = TypedBuffer::new(device.create_buffer(&BufferDescriptor{
            label: Some("Model Cache Vertex Buffer"),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            size: vertex_buffer_size as BufferAddress,
            mapped_at_creation: false,
        }));

        return Self {
            mesh_references: SlotMap::with_capacity_and_key(TEXTURED_MESH_REFERENCE_START_CAPACITY),
            indices,
            vertices,
        }
    }

    fn create_entry(&mut self,queue: &Queue,gltf_data: &[u8]) -> Result<Vec<MeshletRange>,ModelError> {
        let (document, buffers, _) = match gltf::import_slice(gltf_data) {
            Ok(value) => value,
            Err(error) => {
                return Err(ModelError::GltfParseFailure(format!("{}",error)));
            },
        };

        //todo... find correct mesh (name matching)
        let Some(model_mesh) = find_model_mesh(&document) else {
            return Err(ModelError::NoMeshes)
        };

        //todo... find correct collision mesh

        //todo... find vis portals/cell bounds

        let mut buffer: Vec<MeshletRange> = Vec::with_capacity(model_mesh.primitives().len());

        for primitive in model_mesh.primitives() {
            match self.import_render_primitive(&buffers,queue,primitive) {
                Ok(value) => {
                    buffer.push(value)
                },
                Err(error) => {
                    return Err(error);
                },
            }
        }

        Ok(buffer)
    }

    pub fn get_index_buffer_slice(&self) -> BufferSlice<'_> {
        self.indices.get_buffer().slice(..)
    }

    pub fn get_vertex_buffer_slice(&self) -> BufferSlice<'_> {
        self.vertices.get_buffer().slice(..)
    }

    pub fn get_vertex_buffer(&self) -> &Buffer {
        self.vertices.get_buffer()
    }

    pub fn get_index_buffer(&self) -> &Buffer {
        self.indices.get_buffer()
    }

    /// Geometry feedback from the mesh cache
    /// 
    /// Reroute back to the mesh cache to provide the meshlets with texture information
    pub fn insert_geometry(&mut self,queue: &Queue,gltf_data: &[u8]) -> Result<Vec<MeshletRange>,ModelError> {
        self.create_entry(queue,gltf_data)
    }

    pub fn create_textured_mesh_reference(&mut self,mesh: Vec<TexturedMeshlet>) -> TexturedMeshReference {
        self.mesh_references.insert(mesh)
    }

    pub fn get_textured_mesh_ref<'a>(&'a self,reference: TexturedMeshReference) -> &'a [TexturedMeshlet] {
        match self.mesh_references.get(reference) {
            Some(value) => value,
            None => &[],
        }
    }
}
