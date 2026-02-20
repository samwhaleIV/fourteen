use super::*;

impl TextPipeline {
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
            label: Some("Text Pipeline Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("text_pipeline.wgsl").into())
        });

        let render_pipeline_layout = &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Text Pipeline Render Layout"),
            bind_group_layouts: &[
                texture_layout,
                uniform_layout,
            ],
            push_constant_ranges: &[]
        });

        let pipeline_creator = pipeline_creator::PipelineCreator {
            graphics_provider,
            render_pipeline_layout,
            shader,
            vertex_buffer_layout: &[
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
            label: "Text Pipeline",
        };
        let pipelines = pipeline_creator.create_variants();

        let vertices = [  
            GlyphVertex { position: [-0.5,-0.5] },
            GlyphVertex { position: [-0.5, 0.5] },
            GlyphVertex { position: [0.5,-0.5] },
            GlyphVertex { position: [0.5, 0.5] }
        ];

        let indices: [u32;Self::INDEX_BUFFER_SIZE as usize] = [
            0,1,2,
            2,1,3
        ];

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor{
            label: Some("Text Pipeline Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX
        });

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor{
            label: Some("Text Pipeline Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX
        });

        let instance_buffer = DoubleBuffer::new(
            device.create_buffer(&BufferDescriptor{
                label: Some("Text Pipeline Instance Buffer"),
                size: TConfig::TEXT_PIPELINE_BUFFER_SIZE as u64,
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
