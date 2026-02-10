use super::super::prelude::*;

pub struct Pipeline3D {
    pipeline: RenderPipeline,
    instance_buffer: DoubleBuffer<ModelInstance>,
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
            label: Some("Pipeline 3D Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/pipeline3D.wgsl").into())
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline 3D Render Layout"),
            bind_group_layouts: &[
                &shared_pipeline_set.texture_layout,
                &shared_pipeline_set.uniform_layout,
            ],
            push_constant_ranges: &[]
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Pipeline 3D"),
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
        todo!();
    }

    fn reset_buffers(&mut self) {
        self.instance_buffer.reset();
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
}

#[repr(C)]
#[derive(Copy,Clone,Debug,Default,Pod,Zeroable)]
pub struct ModelInstance {
    pub transform_0: [f32;4],
    pub transform_1: [f32;4],
    pub transform_2: [f32;4],
    pub transform_3: [f32;4],
    pub diffuse_color: [u8;4],
    pub lightmap_color: [u8;4]
}

#[non_exhaustive]
struct ATTR;

impl ATTR {
    pub const DIFFUSE_UV: u32 = 0;
    pub const LIGHTMAP_UV: u32 = 1;
    pub const POSITION: u32 = 2;
    pub const TRANSFORM_0: u32 = 3;
    pub const TRANSFORM_1: u32 = 4;
    pub const TRANSFORM_2: u32 = 5;
    pub const TRANSFORM_3: u32 = 6;
    pub const DIFFUSE_COLOR: u32 = 7;
    pub const LIGHTMAP_COLOR: u32 = 8;
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
    const ATTRS: [wgpu::VertexAttribute;6] = wgpu::vertex_attr_array![
        ATTR::TRANSFORM_0 => Float32x4,
        ATTR::TRANSFORM_1 => Float32x4,
        ATTR::TRANSFORM_2 => Float32x4,
        ATTR::TRANSFORM_3 => Float32x4,
        ATTR::DIFFUSE_COLOR => Unorm8x4,
        ATTR::LIGHTMAP_COLOR => Unorm8x4,
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
            transform_0: value.transform.x.into(),
            transform_1: value.transform.y.into(),
            transform_2: value.transform.z.into(),
            transform_3: value.transform.w.into(),
            diffuse_color: value.diffuse_color.decompose(),
            lightmap_color: value.lightmap_color.decompose(),
        }
    }
}

impl From<DrawData3D> for ModelInstance {
    fn from(value: DrawData3D) -> Self {
        ModelInstance::from(&value)
    }
}

pub struct FrameRenderPass3D<TFrame> {
    frame: TFrame
}

impl<TFrame> FrameRenderPass<TFrame> for FrameRenderPass3D<TFrame>
where 
    TFrame: MutableFrame
{
    fn create(frame: TFrame) -> Self {
        return Self {
            frame
        }
    }
    
    fn begin_hardware_pass(self,render_pass: &mut RenderPass,render_pipelines: &mut RenderPipelines) -> TFrame {
        todo!()
    }
    
    fn get_frame(&self) -> &TFrame {
        return &self.frame;
    }
    
    fn get_frame_mut(&mut self) -> &mut TFrame {
        return &mut self.frame;
    }
}

impl<TFrame> FrameRenderPass3D<TFrame>
where 
    TFrame: MutableFrame
{
    // TODO
}
