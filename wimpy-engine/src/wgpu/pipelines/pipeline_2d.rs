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
    pipelines::*,
    shader_definitions::{
        CameraUniform,
        QuadInstance,
        Vertex
    }
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
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &shared_pipeline_set.texture_layout, // This is where the 'texture bind group' is set to bind group index '0'
                &shared_pipeline_set.uniform_layout, // This is where the 'uniform bind group' is set to bind group index '1'
            ],
            push_constant_ranges: &[]
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[
                    Vertex::get_buffer_layout(), // Once again, even though it's stupid, this is where 'VERTEX_BUFFER_INDEX' is defined ... implicitly
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
            Vertex { position: [-0.5,-0.5] }, // Top Left     0
            Vertex { position: [-0.5, 0.5] }, // Bottom Left  1
            Vertex { position: [0.5,-0.5] },  // Top Right    2
            Vertex { position: [0.5, 0.5] }   // Bottom Right 3
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

        let instance_buffer = device.create_buffer(&BufferDescriptor{
            label: Some("Instance Buffer"),
            size: (size_of::<QuadInstance>() * TConfig::INSTANCE_CAPACITY) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let instance_buffer = DoubleBuffer::with_capacity(TConfig::INSTANCE_CAPACITY,instance_buffer);

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
