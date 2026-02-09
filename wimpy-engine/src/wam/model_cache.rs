use std::{
    marker::PhantomData, num::NonZero, ops::Range
};

use bytemuck::{Pod, Zeroable};
use gltf::{
    Document,
    Mesh,
    Primitive,
    buffer::Data, mesh::util::{ReadIndices, ReadPositions}
};

use rapier3d::{math::Vec3, prelude::{
    Ball,
    Capsule,
    Cuboid,
    TriMesh
}};

use slotmap::{
    SecondaryMap,
    SlotMap
};

use wgpu::{
    Buffer,
    BufferDescriptor,
    Queue, QueueWriteBufferView,
};

use crate::wgpu::{
    GraphicsProvider,
    ModelVertex
};

slotmap::new_key_type! {
    pub struct ModelCacheReference;
}

#[derive(Debug)]
pub enum CollisionShape {
    TriMesh(TriMesh),
    //Other shapes not yet implemented
    Cuboid(Cuboid),
    Sphere(Ball),
    Capsule(Capsule)
}

pub struct TypedBuffer<T> {
    value: Buffer,
    length: usize,
    phantom: PhantomData<T>,
}

struct BufferWriteFrame {
    view: QueueWriteBufferView,
    stride: usize
}

impl<T> TypedBuffer<T>
where
    T: Pod + Zeroable
{
    fn new(buffer: Buffer) -> Self {
        return TypedBuffer {
            value: buffer,
            length: 0,
            phantom: Default::default()
        }
    }

    fn get_view(&self,queue: &Queue,length: usize) -> Option<BufferWriteFrame> {
        let stride = length * size_of::<T>();
        match queue.write_buffer_with(
            &self.value,
            (self.length * size_of::<T>()) as u64,
            match NonZero::new(stride as u64) {
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

    fn write(&mut self,queue: &Queue,values: &[T]) -> Option<Range<usize>> {
        let Some(mut frame) = self.get_view(queue,values.len()) else {
            return None;
        };

        frame.view.copy_from_slice(bytemuck::cast_slice(&values));

        let start = self.length;
        self.length += frame.stride;

        return Some(Range {
            start,
            end: self.length
        })
    }

    pub fn get_buffer(&self) -> &Buffer {
        return &self.value;
    }
}

pub struct RenderBuffer {
    index_buffer: TypedBuffer<u32>,
    vertex_buffer: TypedBuffer<ModelVertex>,
}

pub struct ModelCache {
    render_buffer: RenderBuffer,
    entries: SlotMap<ModelCacheReference,RenderBufferReference>,
    collision_shapes: SecondaryMap<ModelCacheReference,CollisionShape>,
}

#[derive(Debug)]
pub struct RenderBufferReference {
    pub index_range: Range<u32>,
    pub base_vertex: i32,
}

pub enum ModelImportError {
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

struct PrimitiveSet<'a> {
    render: Option<Primitive<'a>>,
    collision: Option<Primitive<'a>>
}

enum PrimitiveType {
    Invalid,
    CanRender,
    CannotRender
}

impl PrimitiveType {
    fn classify(primitive: &Primitive) -> PrimitiveType {
        let mut has_position = false;
        let mut can_render = false;

        use gltf::Semantic::*;

        for (semantic, _) in primitive.attributes() {
            match semantic {
                Positions => has_position = true,
                Normals | TexCoords(_)| Colors(_) | Tangents => can_render = true,
                _ => {}
            }
        }

        return match (has_position,can_render) {
            (true, true) => PrimitiveType::CanRender,
            (true, false) => PrimitiveType::CannotRender,
            (false, true) => PrimitiveType::Invalid,
            (false, false) => PrimitiveType::Invalid,
        };
    }
}

impl<'a> PrimitiveSet<'a> {
    fn evaluate_mesh(mesh: &'a Mesh) -> Self {
        let mut render_primitive: Option<Primitive> = None;
        let mut collision_primitive: Option<Primitive> = None;

        for primitive in mesh.primitives() {
            match PrimitiveType::classify(&primitive) {
                PrimitiveType::CanRender => {
                    if render_primitive.is_none() {
                        render_primitive = Some(primitive);
                    }
                },
                PrimitiveType::CannotRender => {
                    if collision_primitive.is_none() {
                        collision_primitive = Some(primitive);
                    }
                },
                PrimitiveType::Invalid => continue,
            }
            if render_primitive.is_some() && collision_primitive.is_some() {
                break;
            }
        }

        return Self {
            render: render_primitive,
            collision: collision_primitive
        };
    }
}

impl RenderBuffer {
    fn import_render_primitive(
        &mut self,
        buffers: &Vec<Data>,
        queue: &Queue,
        primitive: Primitive,
    ) -> Result<RenderBufferReference,ModelImportError> {
        let reader = primitive.reader(|buffer|Some(&buffers[buffer.index()]));

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

        let mut vertices: Vec<ModelVertex> = Vec::with_capacity(positions.len());

        for i in 0..positions.len() {
            let vertex = ModelVertex {
                diffuse_uv: diffuse_uvs[i],
                lightmap_uv:lightmap_uvs[i],
                position: positions[i],
            };
            vertices[i] = vertex;
        }

        let Some(vertex_range) = self.vertex_buffer.write(queue,&vertices) else {
            return Err(ModelImportError::VertexBufferWriteFailure);
        };

        let Some(index_range) = self.index_buffer.write(queue,&indices) else {
            return Err(ModelImportError::IndexBufferWriteFailure);
        };

        let entry = RenderBufferReference {
            index_range: Range {
                start: index_range.start as u32,
                end: index_range.end as u32
            },
            base_vertex: vertex_range.end as i32,
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

fn read_indices_for_trimesh(values: ReadIndices<'_>) -> Vec<[u32;3]> {
    values.into_u32()
        .collect::<Vec<u32>>()
        .chunks_exact(3)
        .map(|triangle|[
            triangle[0],
            triangle[1],
            triangle[2]
        ]).collect()
}

fn read_vertices_for_trimesh(values: ReadPositions<'_>) -> Vec<Vec3> {
    values.map(|vertex|Vec3 {
        x: vertex[0],
        y: vertex[1],
        z: vertex[2],
    }).collect()
}

impl ModelCache {
    pub fn create(graphics_provider: &GraphicsProvider,vertex_buffer_size: usize,index_buffer_size: usize) -> Self {
        let device = graphics_provider.get_device();

        let index_buffer = TypedBuffer::new(device.create_buffer(&BufferDescriptor{
            label: Some("Model Cache Index Buffer"),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            size: index_buffer_size as u64,
            mapped_at_creation: false,
        }));

        let vertex_buffer = TypedBuffer::new(device.create_buffer(&BufferDescriptor{
            label: Some("Model Cache Vertex Buffer"),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            size: vertex_buffer_size as u64,
            mapped_at_creation: false,
        }));

        return Self {
            render_buffer: RenderBuffer {
                index_buffer,
                vertex_buffer,
            },
            entries: SlotMap::with_key(),
            collision_shapes: SecondaryMap::new(),
        }
    }

    fn get_collision_primitive(
        buffers: &Vec<Data>,
        primitive: Primitive
    ) -> Result<TriMesh,ModelImportError> {
        let reader = primitive.reader(|buffer|Some(&buffers[buffer.index()]));

        let vertices: Vec<Vec3> = match reader.read_positions() {
            Some(values) => read_vertices_for_trimesh(values),
            None => return Err(ModelImportError::MissingPositions),
        };

        let indices: Vec<[u32;3]> = match reader.read_indices() {
            Some(values) => read_indices_for_trimesh(values),
            None => return Err(ModelImportError::MissingIndices),
        };
        
        let mesh = match TriMesh::new(vertices,indices) {
            Ok(value) => value,
            Err(error) => return Err(ModelImportError::TriMeshCreationFailure(format!("{:?}",error))),
        };

        return Ok(mesh);
    }

    pub fn create_entry(&mut self,graphics_provider: &GraphicsProvider,gltf_data: &[u8]) -> Result<ModelCacheReference,ModelImportError> {
        let (document, buffers, _) = match gltf::import_slice(gltf_data) {
            Ok(value) => value,
            Err(error) => {
                return Err(ModelImportError::GltfParseFailure(format!("{}",error)));
            },
        };

        let Some(mesh) = find_model_mesh(&document) else {
            return Err(ModelImportError::NoMeshes)
        };

        let primitive_set = PrimitiveSet::evaluate_mesh(&mesh);

        let Some(render_primitive) = primitive_set.render else {
            return Err(ModelImportError::NoRenderPrimitive);
        };

        let render_buffer_reference = self.render_buffer.import_render_primitive(
            &buffers,
            graphics_provider.get_queue(),
            render_primitive
        )?;
        
        let model_cache_reference = self.entries.insert(render_buffer_reference);

        if let Some(primitive) = primitive_set.collision {
            let trimesh = Self::get_collision_primitive(&buffers,primitive)?;
            self.collision_shapes.insert(model_cache_reference,CollisionShape::TriMesh(trimesh));
        }

        return Ok(model_cache_reference);
    }
}
