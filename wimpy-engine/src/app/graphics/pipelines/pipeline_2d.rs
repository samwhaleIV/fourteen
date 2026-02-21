use wgpu::*;
use wgpu::util::{BufferInitDescriptor,DeviceExt};
use std::ops::Range;
use bytemuck::{Pod,Zeroable};
use crate::{WimpyColor, WimpyRect};
use crate::app::graphics::{*,constants::*};
use super::core::*;

pub struct Pipeline2D {
    pipelines: PipelineVariants,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    instance_buffer: DoubleBuffer<QuadInstance>,
}

const VERTEX_BUFFER_INDEX: u32 = 0;
const INSTANCE_BUFFER_INDEX: u32 = 1;
const INDEX_BUFFER_SIZE: u32 = 6;
const TEXTURE_BIND_GROUP_INDEX: u32 = 0;
const UNIFORM_BIND_GROUP_INDEX: u32 = 1;

impl Pipeline2D {

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
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/pipeline2D.wgsl").into())
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

        let pipeline_creator = PipelineCreator {
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

        let indices: [u32;INDEX_BUFFER_SIZE as usize] = [
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
                size: TConfig::INSTANCE_BUFFER_SIZE_2D as BufferAddress,
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

pub struct Pipeline2DPass<'a,'frame> {
    context: &'a mut RenderPassContext<'frame>,
    render_pass: &'a mut RenderPass<'frame>,
    needs_sampler_update: bool,
    sampler_mode: SamplerMode,
    current_sampling_frame: FrameCacheReference,
}

pub struct DrawData2D {
    pub destination: WimpyRect,
    pub source: WimpyRect,
    pub color: WimpyColor,
    pub rotation: f32
}

impl Default for DrawData2D {
    fn default() -> Self {
        Self {
            destination: WimpyRect::ONE,
            source: WimpyRect::ONE,
            color: WimpyColor::WHITE,
            rotation: 0.0
        }
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

impl<'a,'frame> PipelinePass<'a,'frame> for Pipeline2DPass<'a,'frame> {
    fn create(
        frame: &'frame impl MutableFrame,
        render_pass: &'a mut RenderPass<'frame>,
        context: &'a mut RenderPassContext<'frame>
    ) -> Self {
        let pipeline_2d = context.get_2d_pipeline();

        render_pass.set_pipeline(pipeline_2d.pipelines.select(frame));

        render_pass.set_index_buffer(
            pipeline_2d.index_buffer.slice(..),
            wgpu::IndexFormat::Uint32
        ); // Index Buffer

        render_pass.set_vertex_buffer(
            VERTEX_BUFFER_INDEX,
            pipeline_2d.vertex_buffer.slice(..)
        ); // Vertex Buffer

        render_pass.set_vertex_buffer(
            INSTANCE_BUFFER_INDEX,
            pipeline_2d.instance_buffer.get_output_buffer().slice(..)
        ); // Instance Buffer

        let transform = TransformUniform::create_ortho(frame.size());
        let uniform_buffer_range = context.get_shared_mut().get_uniform_buffer().push(transform);
        let dynamic_offset = uniform_buffer_range.start * UNIFORM_BUFFER_ALIGNMENT;

        render_pass.set_bind_group(
            UNIFORM_BIND_GROUP_INDEX,
            context.get_shared().get_uniform_bind_group(),
            &[dynamic_offset as u32]
        );

        let current_sampling_frame = context.textures.transparent_black.get_ref();

        return Self {
            context,
            render_pass,
            needs_sampler_update: true,
            sampler_mode: SamplerMode::NearestWrap,
            current_sampling_frame: current_sampling_frame
        }
    }
}

impl Pipeline2DPass<'_,'_> {
    fn update_sampler_if_needed(&mut self,reference: FrameCacheReference) -> Result<(),()> {
        if !(self.needs_sampler_update || self.current_sampling_frame != reference) {
            return Err(());
        }

        self.current_sampling_frame = reference;
        self.needs_sampler_update = false;

        return match self.context.frame_cache.get(reference) {
            Ok(texture_container) => {
                self.context.set_texture_bind_group(
                    TEXTURE_BIND_GROUP_INDEX,
                    &mut self.render_pass,
                    &BindGroupCacheIdentity::SingleChannel {
                    ch_0: BindGroupChannelConfig {
                        mode: self.sampler_mode,
                        texture: texture_container,
                    }
                });
                Ok(())
            },
            Err(error) => {
                log::warn!("Unable to get texture container for sampler; the texture container cannot be found: {:?}",error);
                Err(())
            }
        };
    }

    pub fn draw(&mut self,frame_reference: &impl FrameReference,draw_data: &[DrawData2D]) {
        let reference = frame_reference.get_ref();
        if let Err(()) = self.update_sampler_if_needed(reference) {
            return;
        }

        let uv_scale = frame_reference.get_uv_scale();

        let range = self.context.get_2d_pipeline_mut().instance_buffer.push_set(draw_data.iter().map(|value|{
            let dst = value.destination.origin_top_left_to_center();
            let src = value.source * uv_scale;
            QuadInstance {
                position: dst.position.into(),
                size: dst.size.into(),
                uv_position: src.position.into(),
                uv_size: src.size.into(),
                color: value.color.into(),
                rotation: value.rotation
            }
        }));

        self.render_pass.draw_indexed(0..INDEX_BUFFER_SIZE,0,Range {
            start: range.start as u32,
            end: range.end as u32,
        });
    }

    pub fn set_sampler_mode(&mut self,sampler_mode: SamplerMode) {
        if self.sampler_mode != sampler_mode {
            self.sampler_mode = sampler_mode;
            self.needs_sampler_update = true;
        }
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
