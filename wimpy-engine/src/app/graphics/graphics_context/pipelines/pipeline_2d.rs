use super::*;

pub struct Pipeline2D {
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    instance_buffer: DoubleBuffer<QuadInstance>,
}

impl Pipeline2D {
    pub const VERTEX_BUFFER_INDEX: u32 = 0;
    pub const INSTANCE_BUFFER_INDEX: u32 = 1;
    pub const INDEX_BUFFER_SIZE: u32 = 6;

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
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/pipeline2D.wgsl").into())
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
            instance_buffer
        }
    }

    pub fn write_quad(&mut self,render_pass: &mut RenderPass,draw_data: &DrawData2D) {
        let range = self.instance_buffer.push_convert(draw_data.into());
        render_pass.draw_indexed(0..Self::INDEX_BUFFER_SIZE,0,downcast_range(range));
    }

    pub fn write_quad_set(&mut self,render_pass: &mut RenderPass,draw_data: &[DrawData2D]) {
        let range = self.instance_buffer.push_convert_all(draw_data);
        render_pass.draw_indexed(0..Self::INDEX_BUFFER_SIZE,0,downcast_range(range));
    }
}

impl PipelineController for Pipeline2D {
    fn write_dynamic_buffers(&mut self,queue: &Queue) {
        self.instance_buffer.write_out(queue);
    }
    
    fn reset_pipeline_state(&mut self) {
        self.instance_buffer.reset();
    }
}

#[repr(C)]
#[derive(Copy,Clone,Debug,Default,Pod,Zeroable)]
pub struct QuadVertex {
    pub position: [f32;2],
}

#[repr(C)]
#[derive(Copy,Clone,Debug,Default,Pod,Zeroable)]
pub struct QuadInstance {
    pub position: [f32;2],
    pub size: [f32;2],
    pub uv_position: [f32;2],
    pub uv_size: [f32;2],
    pub color: [u8;4],
    pub rotation: f32,
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
        ATTR::COLOR => Unorm8x4,
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
            color: value.color.decompose(),
            rotation: value.rotation
        }
    }
}

impl From<DrawData2D> for QuadInstance {
    fn from(value: DrawData2D) -> Self {
        QuadInstance::from(&value)
    }
}

pub struct FrameRenderPass2D<TFrame> {
    frame: TFrame,
}

impl<TFrame> FrameRenderPass<TFrame> for FrameRenderPass2D<TFrame>
where 
    TFrame: MutableFrame
{
    fn get_frame(&self) -> &TFrame {
        return &self.frame;
    }
    fn get_frame_mut(&mut self) -> &mut TFrame {
        return &mut self.frame;
    }
   
    fn begin_render_pass(self,render_pass: &mut RenderPass,pipeline_view: &mut RenderPassView) -> TFrame {
        let pipeline_2d = pipeline_view.get_2d_pipeline();

        render_pass.set_pipeline(&pipeline_2d.render_pipeline); 

        render_pass.set_index_buffer(
            pipeline_2d.index_buffer.slice(..),
            wgpu::IndexFormat::Uint32
        ); // Index Buffer

        render_pass.set_vertex_buffer(
            Pipeline2D::VERTEX_BUFFER_INDEX,
            pipeline_2d.vertex_buffer.slice(..)
        ); // Vertex Buffer

        render_pass.set_vertex_buffer(
            Pipeline2D::INSTANCE_BUFFER_INDEX,
            pipeline_2d.instance_buffer.get_output_buffer().slice(..)
        ); // Instance Buffer


        let shared_pipeline = pipeline_view.get_shared_pipeline_mut();

        let transform = MatrixTransformUniform::create_ortho(self.size());
        let uniform_buffer_range = shared_pipeline.get_uniform_buffer().push(transform);
        let dynamic_offset = uniform_buffer_range.start * UNIFORM_BUFFER_ALIGNMENT;

        render_pass.set_bind_group(
            UNIFORM_BIND_GROUP_INDEX,
            shared_pipeline.get_uniform_bind_group(),
            &[dynamic_offset as u32]
        );
        self.frame
    }
}

impl<TFrame> FrameRenderPass2D<TFrame>
where 
    TFrame: MutableFrame
{
    pub fn draw(&mut self,frame_reference: &impl FrameReference,draw_data: DrawData2D) {
        self.get_frame_mut().push_command(
            FrameCommand::Draw2D {
                reference: frame_reference.get_cache_reference(),
                draw_data: DrawData2D {
                    destination: draw_data.destination,
                    source: draw_data.source.multiply_2d(frame_reference.get_output_uv_size()),
                    color: draw_data.color,
                    rotation: draw_data.rotation
                }
            }
        );
    }
}
