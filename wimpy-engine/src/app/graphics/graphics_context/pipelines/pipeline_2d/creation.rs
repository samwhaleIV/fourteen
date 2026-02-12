use super::*;

impl Pipeline2D {
    pub fn create<TConfig>(
        graphics_provider: &GraphicsProvider,
        shared_pipeline: &SharedPipeline
    ) -> Self
    where 
        TConfig: GraphicsContextConfig
    {
        let device = graphics_provider.get_device();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Pipeline 2D Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("pipeline2D.wgsl").into())
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline 2D Render Layout"),
            bind_group_layouts: &[
                // This is where the 'texture bind group' is set to bind group index '0'
                &shared_pipeline.get_texture_layout(),
                // This is where the 'uniform bind group' is set to bind group index '1'
                &shared_pipeline.get_uniform_layout(),
            ],
            push_constant_ranges: &[]
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Pipeline 2D"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[
                    // Once again, even though it's stupid, this is where 'VERTEX_BUFFER_INDEX' is defined ... implicitly
                    QuadVertex::get_buffer_layout(),
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
            // TODO: enable depth stencil
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
            render_pipeline: pipeline,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            command_buffer_pool: VecPool::new()
        }
    }
}
