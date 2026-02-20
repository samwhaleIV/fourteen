use super::*;

impl Pipeline2D {
    pub const VERTEX_BUFFER_INDEX: u32 = 0;
    pub const INSTANCE_BUFFER_INDEX: u32 = 1;
    pub const INDEX_BUFFER_SIZE: u32 = 6;

    pub fn create<TConfig>(
        graphics_provider: &GraphicsProvider,
        texture_layout: &BindGroupLayout,
        uniform_layout: &BindGroupLayout,
    ) -> Self
    where
        TConfig: GraphicsContextConfig
    {
        let device = graphics_provider.get_device();

        let shader = &device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Pipeline 2D Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("pipeline2D.wgsl").into())
        });

        let render_pipeline_layout = &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline 2D Render Layout"),
            bind_group_layouts: &[
                // This is where the 'texture bind group' is set to bind group index '0'
                texture_layout,
                // This is where the 'uniform bind group' is set to bind group index '1'
                uniform_layout,
            ],
            push_constant_ranges: &[]
        });

        let pipeline_creator = pipeline_creator::PipelineCreator {
            graphics_provider,
            render_pipeline_layout,
            shader,
            vertex_buffer_layout: &[
                // Once again, even though it's stupid, this is where 'VERTEX_BUFFER_INDEX' is defined ... implicitly
                QuadVertex::get_buffer_layout(),
                QuadInstance::get_buffer_layout()
            ],
            primitive_state: &wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false
            },
            label: "Pipeline 2D",
        };
        let pipelines = pipeline_creator.create_variants();
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
            label: Some("Pipeline 2D Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX
        });

        // Investigate if vertex buffer can be put at the start of the instance buffer
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor{
            label: Some("Pipeline 2D Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX
        });

        let instance_buffer = DoubleBuffer::new(
            device.create_buffer(&BufferDescriptor{
                label: Some("Pipeline 2D Instance Buffer"),
                size: TConfig::INSTANCE_BUFFER_SIZE_2D as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        );

        return Self {
            pipelines,
            vertex_buffer,
            index_buffer,
            instance_buffer,
        }
    }
}
