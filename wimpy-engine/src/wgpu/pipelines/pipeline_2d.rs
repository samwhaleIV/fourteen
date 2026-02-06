use wgpu::{
    *,
    util::{
        BufferInitDescriptor,
        DeviceExt
    }
};

use crate::wgpu::{
    GraphicsContextConfig,
    GraphicsProvider,
    DoubleBuffer,
    pipelines::*
};

pub struct Pipeline2D {
    pub pipeline: RenderPipeline,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub instance_buffer: DoubleBuffer<QuadInstance>,
}

impl Pipeline2D {
    pub const VERTEX_BUFFER_INDEX: u32 = 0;
    pub const INSTANCE_BUFFER_INDEX: u32 = 1;
    pub const INDEX_BUFFER_SIZE: u32 = 6;

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
                &shared_pipeline_set.texture_layout, // This is where the 'texture bind group' is set to bind group index '0'
                &shared_pipeline_set.uniform_layout, // This is where the 'uniform bind group' is set to bind group index '1'
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
                    QuadVertex::get_buffer_layout(), // Once again, even though it's stupid, this is where 'VERTEX_BUFFER_INDEX' is defined ... implicitly
                    QuadInstance::get_buffer_layout()
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
                cull_mode: None,
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

/*
        Triangle list should generate 0-1-2 2-1-3 in CCW

                        0---2
                        |  /|
                        | / |
                        |/  |
                        1---3
*/
        let vertices = [  
            QuadVertex { position: [-0.5,-0.5] }, // Top Left     0
            QuadVertex { position: [-0.5, 0.5] }, // Bottom Left  1
            QuadVertex { position: [0.5,-0.5] },  // Top Right    2
            QuadVertex { position: [0.5, 0.5] }   // Bottom Right 3
        ];

        let indices: [u32;Self::INDEX_BUFFER_SIZE as usize] = [
            0,1,2,
            2,1,3
        ];

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor{
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX
        });

        // Investigate if vertex buffer can be put at the start of the instance buffer
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor{
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX
        });

        let instance_buffer = DoubleBuffer::new(
            device.create_buffer(&BufferDescriptor{
                label: Some("Instance Buffer"),
                size: TConfig::INSTANCE_BUFFER_SIZE_2D as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        );

        return Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            instance_buffer
        }
    }
}

impl RenderPassController for Pipeline2D {
    fn begin(
        &mut self,
        render_pass: &mut RenderPass,
        shared_pipeline: &mut SharedPipelineSet,
        uniform: CameraUniform
    ) {
        render_pass.set_pipeline(&self.pipeline); 

        render_pass.set_index_buffer(
            self.index_buffer.slice(..),
            wgpu::IndexFormat::Uint32
        ); // Index Buffer

        render_pass.set_vertex_buffer(
            Self::VERTEX_BUFFER_INDEX,
            self.vertex_buffer.slice(..)
        ); // Vertex Buffer

        render_pass.set_vertex_buffer(
            Self::INSTANCE_BUFFER_INDEX,
            self.instance_buffer.get_output_buffer().slice(..)
        ); // Instance Buffer

        let uniform_buffer_range = shared_pipeline.uniform_buffer.push(uniform);
        let dynamic_offset = uniform_buffer_range.start * UNIFORM_BUFFER_ALIGNMENT;

        render_pass.set_bind_group(
            UNIFORM_BIND_GROUP_INDEX,
            &shared_pipeline.uniform_bind_group,
            &[dynamic_offset as u32]
        ); // Uniform Buffer Bind Group
    }

    fn write_buffers(&mut self,queue: &Queue) {
        self.instance_buffer.write_out(queue);
    }

    fn reset_buffers(&mut self) {
        self.instance_buffer.reset();
    }
    
    fn select_and_begin(
        render_pass: &mut RenderPass,
        render_pipelines: &mut super::RenderPipelines,
        uniform: CameraUniform
    ) {
        render_pipelines.pipeline_2d.begin(render_pass,&mut render_pipelines.shared,uniform);
    }
}

#[repr(C)]
#[derive(Copy,Clone,Debug,Default,Pod,Zeroable)]
pub struct QuadVertex {
    pub position: [f32;2],
    //_padding: [f32;2]
}

#[repr(C)]
#[derive(Copy,Clone,Debug,Default,Pod,Zeroable)]
pub struct QuadInstance { //Aligned to 64
    pub position: [f32;2],
    pub size: [f32;2],
    pub uv_position: [f32;2],
    pub uv_size: [f32;2],
    pub color: [f32;4],
    pub rotation: f32,
    pub _padding: [f32;3]
}

#[non_exhaustive]
struct ATTR;

impl ATTR {
    pub const VERTEX_POSITION: u32 = 0;
    pub const INSTANCE_POSITION: u32 = 1;
    pub const SIZE: u32 = 2;
    pub const UV_POS: u32 = 3;
    pub const UV_SIZE: u32 = 4;
    pub const COLOR: u32 = 5;
    pub const ROTATION: u32 = 6;
}

impl QuadVertex {
    const ATTRS: [wgpu::VertexAttribute;1] = wgpu::vertex_attr_array![
        ATTR::VERTEX_POSITION => Float32x2,
    ];

    pub fn get_buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        return wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRS,
        }
    }
}

impl QuadInstance {
    const ATTRS: [wgpu::VertexAttribute;6] = wgpu::vertex_attr_array![
        ATTR::INSTANCE_POSITION => Float32x2,
        ATTR::SIZE => Float32x2,
        ATTR::UV_POS => Float32x2,
        ATTR::UV_SIZE => Float32x2,
        ATTR::COLOR => Float32x4,
        ATTR::ROTATION => Float32,
    ];

    pub fn get_buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        return wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRS,
        }
    }
}

impl<'a> From<&'a DrawData2D> for QuadInstance {
    fn from(value: &'a DrawData2D) -> Self {
        let area = value.destination.to_center_encoded();
        return QuadInstance {
            position: [
                area.x,
                area.y,
            ],
            size: [
                area.width,
                area.height,
            ],
            uv_position: [
                value.source.x,
                value.source.y,
            ],
            uv_size: [
                value.source.width,
                value.source.height,
            ],
            color: value.color.to_float_array(),
            rotation: value.rotation,
            _padding: [0.0,0.0,0.0],
        }
    }
}

impl From<DrawData2D> for QuadInstance {
    fn from(value: DrawData2D) -> Self {
        QuadInstance::from(&value)
    }
}
