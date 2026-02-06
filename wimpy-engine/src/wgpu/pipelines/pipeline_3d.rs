use std::{
    num::NonZero,
    ops::Range
};

use bytemuck::*;

use gltf::{
    Primitive,
    buffer::Data};

use wgpu::{
    Buffer,
    BufferDescriptor,
    Queue,
    RenderPipeline
};

use crate::wgpu::{
    DoubleBuffer,
    DrawData3D,
    GraphicsContextConfig,
    GraphicsProvider,
    pipelines::{
        CameraUniform,
        RenderPassController,
        SharedPipelineSet
    }
};

pub struct Pipeline3D {
    pipeline: RenderPipeline,
    model_cache: ModelCache,
    instance_buffer: DoubleBuffer<ModelInstance>,
}

pub struct ModelCache {
    index_buffer: Buffer,
    vertex_buffer: Buffer,

    index_buffer_length: usize,
    vertex_buffer_length: usize,
}

pub struct ModelCacheEntry {
    index_range: Range<u32>,
    base_vertex: i32,
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

        return Ok(entry);
    }
}

impl Pipeline3D {
    pub fn create<TConfig>(
        graphics_provider: &GraphicsProvider,
        shared_pipeline_set: &SharedPipelineSet
    ) -> Self
    where
        TConfig: GraphicsContextConfig    
    {
        let device = graphics_provider.get_device();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/pipeline2D.wgsl").into())
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("2D Render Pipeline Layout"),
            bind_group_layouts: &[
                &shared_pipeline_set.texture_layout,
                &shared_pipeline_set.uniform_layout,
            ],
            push_constant_ranges: &[]
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("2D Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[
                    ModelVertex::get_buffer_layout(),
                    ModelInstance::get_buffer_layout()
                ]
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: graphics_provider.get_output_format(),
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })]
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false     
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None
        });

        let index_buffer = device.create_buffer(&BufferDescriptor{
            label: Some("Index Buffer"),      
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            size: TConfig::INDEX_BUFFER_SIZE_3D as u64,
            mapped_at_creation: false,
        });

        let vertex_buffer = device.create_buffer(&BufferDescriptor{
            label: Some("Vertex Buffer"),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            size: TConfig::VERTEX_BUFFER_SIZE_3D as u64,
            mapped_at_creation: false,
        });

        let instance_buffer = DoubleBuffer::new(
            device.create_buffer(&BufferDescriptor{
                label: Some("Instance Buffer"),
                size: TConfig::INSTANCE_BUFFER_SIZE_3D as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        );

        return Self {
            pipeline,
            model_cache: ModelCache {
                vertex_buffer,
                index_buffer,
                index_buffer_length: 0,
                vertex_buffer_length: 0,
            },
            instance_buffer,
        }
    }
}

impl RenderPassController for Pipeline3D {
    fn begin(
        &mut self,
        render_pass: &mut wgpu::RenderPass,
        shared_pipeline: &mut SharedPipelineSet,
        uniform: CameraUniform
    ) {
        todo!()
    }

    fn write_buffers(&mut self,queue: &wgpu::Queue) {
        todo!()
    }

    fn reset_buffers(&mut self) {
        todo!()
    }
    
    fn select_and_begin(
        render_pass: &mut wgpu::RenderPass,
        render_pipelines: &mut super::RenderPipelines,
        uniform: CameraUniform
    ) {
        todo!()
    }
}

#[repr(C)]
#[derive(Copy,Clone,Debug,Default,Pod,Zeroable)]
pub struct ModelVertex {
    pub diffuse_uv: [f32;2],
    pub lightmap_uv: [f32;2],
    pub position: [f32;3],
    _padding: [f32;1],
}

#[repr(C)]
#[derive(Copy,Clone,Debug,Default,Pod,Zeroable)]
pub struct ModelInstance {
    pub diffuse_color: [f32;4],
    pub lightmap_color: [f32;4]
}

#[non_exhaustive]
struct ATTR;

impl ATTR {
    pub const DIFFUSE_UV: u32 = 0;
    pub const LIGHTMAP_UV: u32 = 1;
    pub const POSITION: u32 = 2;
    pub const DIFFUSE_COLOR: u32 = 3;
    pub const LIGHTMAP_COLOR: u32 = 4;
}

impl ModelVertex {
    const ATTRS: [wgpu::VertexAttribute;3] = wgpu::vertex_attr_array![
        ATTR::DIFFUSE_UV => Float32x2,
        ATTR::LIGHTMAP_UV => Float32x2,
        ATTR::POSITION => Float32x3
    ];

    pub fn get_buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        return wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRS,
        }
    }
}

impl ModelInstance {
    const ATTRS: [wgpu::VertexAttribute;2] = wgpu::vertex_attr_array![
        ATTR::DIFFUSE_COLOR => Float32x4,
        ATTR::LIGHTMAP_COLOR => Float32x4,
    ];

    pub fn get_buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        return wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRS,
        }
    }
}

impl<'a> From<&'a DrawData3D> for ModelInstance {
    fn from(value: &'a DrawData3D) -> Self {
        return ModelInstance {
            diffuse_color: value.diffuse_color.to_float_array(),
            lightmap_color: value.lightmap_color.to_float_array(),
        }
    }
}

impl From<DrawData3D> for ModelInstance {
    fn from(value: DrawData3D) -> Self {
        ModelInstance::from(&value)
    }
}
