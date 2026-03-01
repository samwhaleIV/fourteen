use glam::Vec3;
use wgpu::*;
use std::ops::Range;
use bytemuck::{Pod,Zeroable};
use crate::{WimpyColorLinear, WimpyVec, app::graphics::*};
use super::core::*;

pub struct LinesPipeline {
    strip_sub_variant: PipelineSet,
    list_sub_variant: PipelineSet,
    line_point_buffer: DoubleBuffer<LineVertex>,
}

const VERTEX_BUFFER_INDEX: u32 = 0;
const UNIFORM_BIND_GROUP_INDEX: u32 = 0;

#[derive(Copy,Clone,PartialEq,Eq)]
enum LinesMode {
    Strip,
    List
}

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

        let strip_sub_variant = PipelineCreator {
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
            label: "Pipeline Lines (Line Strip)",
        }.create_pipeline_set();

        let list_sub_variant = PipelineCreator {
            graphics_provider,
            render_pipeline_layout,
            shader,
            vertex_buffer_layout: &[
                LineVertex::get_buffer_layout(),
            ],
            primitive_state: &PrimitiveState {
                topology: PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false
            },
            label: "Pipeline Lines (Line Strip)",
        }.create_pipeline_set();

        let line_point_buffer = DoubleBuffer::new(
            device.create_buffer(&BufferDescriptor{
                label: Some("Pipeline Lines Vertex Buffer"),
                size: TConfig::LINE_BUFFER_SIZE as BufferAddress,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        );

        return Self {
            line_point_buffer,
            strip_sub_variant,
            list_sub_variant,
        }
    }
}

pub struct LinesPipelinePass<'a,'frame> {
    context: &'a mut RenderPassContext<'frame>,
    render_pass: &'a mut RenderPass<'frame>,
    lines_mode: Option<LinesMode>,
    uniform_reference: UniformReference,
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
        render_pass: &'a mut RenderPass<'frame>,
        context: &'a mut RenderPassContext<'frame>,
        uniform_reference: UniformReference,
    ) -> Self {
        let lines_pipeline = context.get_line_pipeline();

        render_pass.set_vertex_buffer(
            VERTEX_BUFFER_INDEX,
            lines_pipeline.line_point_buffer.get_output_buffer().slice(..)
        );

        return Self {
            context,
            uniform_reference,
            render_pass,
            lines_mode: None,
        }
    }
}

impl LinesPipelinePass<'_,'_> {
    fn draw<I>(&mut self,line_points: I)
    where
        I: Iterator<Item = LineVertex>
    {
        let buffer = &mut self.context.pipelines.get_unique_mut().lines_pipeline.line_point_buffer;
        let start = buffer.len();
        buffer.push_set(line_points);
        let end = buffer.len();
        if start == end {
            return;
        }
        self.render_pass.draw(Range {
            start: start as u32,
            end: end as u32
        },0..1);
    }

    fn set_pipeline(&mut self,mode: LinesMode) {
        if let Some(current_mode) = self.lines_mode && mode == current_mode {
            return;
        }
        self.render_pass.set_pipeline(match mode {
            LinesMode::Strip => {
                &self.context.get_line_pipeline().strip_sub_variant
            },
            LinesMode::List => {
                &self.context.get_line_pipeline().list_sub_variant
            },
        }.select(self.context.variant_key));
        self.context.get_shared().bind_uniform::<UNIFORM_BIND_GROUP_INDEX>(self.render_pass,self.uniform_reference);
        self.lines_mode = Some(mode);
    }

    pub fn draw_strip<I>(&mut self,line_points: I)
    where
        I: IntoIterator,
        I::Item: Into<LineVertex>
    {
        self.set_pipeline(LinesMode::Strip);
        self.draw(line_points.into_iter().map(Into::into));
    }

    pub fn draw_list<I>(&mut self,line_points: I)
    where
        I: IntoIterator,
        I::Item: Into<LineVertex>
    {
        self.set_pipeline(LinesMode::List);
        self.draw(line_points.into_iter().map(Into::into));
    }
}

pub struct LinePoint2D {
    pub point: WimpyVec,
    pub color: WimpyColorLinear
}

pub struct LinePoint3D {
    pub point: Vec3,
    pub color: WimpyColorLinear
}

#[repr(C)]
#[derive(Copy,Clone,Debug,Default,Pod,Zeroable)]
pub struct LineVertex {
    pub position: [f32;3],
    pub color: [f32;4]
}

impl From<LinePoint2D> for LineVertex {
    fn from(value: LinePoint2D) -> Self {
        Self {
            position: [value.point.x,value.point.y,0.0],
            color: value.color.into()
        }
    }
}

impl From<&LinePoint2D> for LineVertex {
    fn from(value: &LinePoint2D) -> Self {
        Self {
            position: [value.point.x,value.point.y,0.0],
            color: value.color.into()
        }
    }
}

impl From<LinePoint3D> for LineVertex {
    fn from(value: LinePoint3D) -> Self {
        Self {
            position: value.point.into(),
            color: value.color.into()
        }
    }
}

impl From<&LinePoint3D> for LineVertex {
    fn from(value: &LinePoint3D) -> Self {
        Self {
            position: value.point.into(),
            color: value.color.into()
        }
    }
}

#[non_exhaustive]
struct ATTR;

impl ATTR {
    pub const VERTEX_POSITION: u32 = 0;
    pub const VERTEX_COLOR: u32 = 1;
}

impl LineVertex {
    const ATTRS: [VertexAttribute;2] = vertex_attr_array![
        ATTR::VERTEX_POSITION => Float32x3,
        ATTR::VERTEX_COLOR => Float32x4
    ];

    pub fn get_buffer_layout<'a>() -> VertexBufferLayout<'a> {
        return VertexBufferLayout {
            array_stride: size_of::<Self>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRS,
        }
    }
}
