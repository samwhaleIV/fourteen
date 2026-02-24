use wgpu::*;
use std::{borrow::Borrow, ops::Range};
use bytemuck::{Pod,Zeroable};
use crate::{WimpyColor, WimpyVec, app::graphics::{constants::*, *}};
use super::core::*;

pub struct LinesPipeline {
    pipelines: PipelineVariants,
    line_point_buffer: DoubleBuffer<LineVertex>,
}

pub const VERTEX_BUFFER_INDEX: u32 = 0;
pub const UNIFORM_BIND_GROUP_INDEX: u32 = 0;

impl LinesPipeline {

    pub fn create<TConfig>(
        graphics_provider: &GraphicsProvider,
        _texture_layout: &BindGroupLayout,
        uniform_layout: &BindGroupLayout,
    ) -> Self
    where
        TConfig: GraphicsContextConfig
    {
        let device = graphics_provider.get_device();

        let shader = &device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Pipeline Lines Shader"),
            source: ShaderSource::Wgsl(include_str!("shaders/lines_pipeline.wgsl").into())
        });

        let render_pipeline_layout = &device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Pipeline Lines Render Layout"),
            bind_group_layouts: &[
                uniform_layout,
            ],
            push_constant_ranges: &[]
        });

        let pipeline_creator = PipelineCreator {
            graphics_provider,
            render_pipeline_layout,
            shader,
            vertex_buffer_layout: &[
                LineVertex::get_buffer_layout(),
            ],
            primitive_state: &PrimitiveState {
                topology: PrimitiveTopology::LineStrip,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false
            },
            label: "Pipeline Lines",
        };
        let pipelines = pipeline_creator.create_variants();

        let line_point_buffer = DoubleBuffer::new(
            device.create_buffer(&BufferDescriptor{
                label: Some("Pipeline Lines Vertex Buffer"),
                size: TConfig::LINE_BUFFER_SIZE as BufferAddress,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        );

        return Self {
            pipelines,
            line_point_buffer
        }
    }
}

pub struct LinesPipelinePass<'a,'frame> {
    context: &'a mut RenderPassContext<'frame>,
    render_pass: &'a mut RenderPass<'frame>,
}

impl PipelineController for LinesPipeline {
    fn write_dynamic_buffers(&mut self,queue: &Queue) {
        self.line_point_buffer.write_out(queue);
    }
    fn reset_pipeline_state(&mut self) {
        self.line_point_buffer.reset();
    }
}

impl<'a,'frame> PipelinePass<'a,'frame> for LinesPipelinePass<'a,'frame> {
    fn create(
        frame: &'frame impl MutableFrame,
        render_pass: &'a mut RenderPass<'frame>,
        context: &'a mut RenderPassContext<'frame>
    ) -> Self {
        let lines_pipeline = context.get_line_pipeline();

        render_pass.set_pipeline(lines_pipeline.pipelines.select(frame));

        render_pass.set_vertex_buffer(
            VERTEX_BUFFER_INDEX,
            lines_pipeline.line_point_buffer.get_output_buffer().slice(..)
        );

        let transform = TransformUniform::create_ortho(frame.size());
        let uniform_buffer_range = context.get_shared_mut().get_uniform_buffer().push(transform);
        let dynamic_offset = uniform_buffer_range.start * UNIFORM_BUFFER_ALIGNMENT;

        render_pass.set_bind_group(
            UNIFORM_BIND_GROUP_INDEX,
            context.get_shared().get_uniform_bind_group(),
            &[dynamic_offset as u32]
        );

        return Self {
            context,
            render_pass,
        }
    }
}

impl LinesPipelinePass<'_,'_> {
    pub fn draw<I>(&mut self,line_points: I)
    where
        I: IntoIterator,
        I::Item: Borrow<LinePoint>
    {
        let buffer = &mut self.context.pipelines.get_unique_mut().lines_pipeline.line_point_buffer;
        let start = buffer.len();
        for line in line_points {
            let line = line.borrow();
            buffer.push(LineVertex {
                position: line.point.into(),
                color: line.color.into()
            });
        }
        let end = buffer.len();
        if start == end {
            return;
        }
        self.render_pass.draw(Range {
            start: start as u32,
            end: end as u32
        },0..1);
    }
}

pub struct LinePoint {
    pub point: WimpyVec,
    pub color: WimpyColor
}

#[repr(C)]
#[derive(Copy,Clone,Debug,Default,Pod,Zeroable)]
pub struct LineVertex {
    pub position: [f32;2],
    pub color: [u8;4]
}

#[non_exhaustive]
struct ATTR;

impl ATTR {
    pub const VERTEX_POSITION: u32 = 0;
    pub const VERTEX_COLOR: u32 = 1;
}

impl LineVertex {
    const ATTRS: [VertexAttribute;2] = vertex_attr_array![
        ATTR::VERTEX_POSITION => Float32x2,
        ATTR::VERTEX_COLOR => Unorm8x4
    ];

    pub fn get_buffer_layout<'a>() -> VertexBufferLayout<'a> {
        return VertexBufferLayout {
            array_stride: size_of::<Self>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRS,
        }
    }
}
