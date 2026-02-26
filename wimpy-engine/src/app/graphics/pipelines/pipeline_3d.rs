use cgmath::{Matrix4,SquareMatrix};
use wgpu::*;
use std::ops::Range;
use bytemuck::{Pod,Zeroable};
use crate::{WimpyColorLinear, app::{graphics::{constants::*, *},wam::ModelData}};
use super::core::*;

pub struct Pipeline3D {
    pipelines: PipelineSet,
    instance_buffer: DoubleBuffer<ModelInstance>,
}

const VERTEX_BUFFER_INDEX: u32 = 0;
const INSTANCE_BUFFER_INDEX: u32 = 1;
const TEXTURE_BIND_GROUP_INDEX: u32 = 0;
const UNIFORM_BIND_GROUP_INDEX: u32 = 1;

impl Pipeline3D {

    pub fn create<TConfig>(
        graphics_provider: &GraphicsProvider,
        texture_layout: &BindGroupLayout,
        uniform_layout: &BindGroupLayout
    ) -> Self
    where
        TConfig: GraphicsContextConfig
    {
        let device = graphics_provider.get_device();

        let shader = &device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Pipeline 3D Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/pipeline3D.wgsl").into())
        });

        let render_pipeline_layout = &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline 3D Render Layout"),
            bind_group_layouts: &[
                texture_layout,
                uniform_layout,
            ],
            push_constant_ranges: &[]
        });

        let pipeline_creator = PipelineCreator {
            graphics_provider,
            render_pipeline_layout,
            shader,
            vertex_buffer_layout: &[
                ModelVertex::get_buffer_layout(),
                ModelInstance::get_buffer_layout()
            ],
            primitive_state: &wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false
            },
            label: "Pipeline 3D",
        };
        let pipelines = pipeline_creator.create_pipeline_set();

        let instance_buffer = DoubleBuffer::new(
            device.create_buffer(&BufferDescriptor{
                label: Some("Pipeline 3D Instance Buffer"),
                size: TConfig::INSTANCE_BUFFER_SIZE_3D as BufferAddress,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        );

        return Self {
            pipelines,
            instance_buffer,
        }
    }
}

struct TextureDrawData {
    diffuse: Option<TextureFrame>,
    lightmap: Option<TextureFrame>,
    diffuse_sampler: SamplerMode,
    strategy: TextureStrategy,
}

impl PipelineController for Pipeline3D {
    fn write_dynamic_buffers(&mut self,queue: &Queue) {
        self.instance_buffer.write_out(queue);
    }
    fn reset_pipeline_state(&mut self) {
        self.instance_buffer.reset();
    }
}

pub struct Pipeline3DPass<'a,'frame> {
    context: &'a mut RenderPassContext<'frame>,
    render_pass: &'a mut RenderPass<'frame>,
    has_transform_bind: bool,
}

impl<'a,'frame> PipelinePass<'a,'frame> for Pipeline3DPass<'a,'frame> {
    fn create(
        frame: &'frame impl MutableFrame,
        render_pass: &'a mut RenderPass<'frame>,
        context: &'a mut RenderPassContext<'frame>
    ) -> Self {
        let pipeline_3d = context.get_3d_pipeline();
        render_pass.set_pipeline(&pipeline_3d.pipelines.select(frame));

        render_pass.set_index_buffer(
            context.model_cache.get_index_buffer_slice(),
            wgpu::IndexFormat::Uint32
        );

        render_pass.set_vertex_buffer(
            VERTEX_BUFFER_INDEX,
            context.model_cache.get_vertex_buffer_slice()
        );

        render_pass.set_vertex_buffer(
            INSTANCE_BUFFER_INDEX,
            pipeline_3d.instance_buffer.get_output_buffer().slice(..)
        );

        return Self {
            //frame_size,
            context,
            render_pass,
            has_transform_bind: false,
        }
    }
}

#[derive(Copy,Clone)]
pub struct DrawData3D {
    pub transform: Matrix4<f32>,
    pub diffuse_color: WimpyColorLinear,
    pub lightmap_color: WimpyColorLinear,
}

impl Default for DrawData3D {
    fn default() -> Self {
        Self {
            transform: Matrix4::identity(),
            diffuse_color: WimpyColorLinear::WHITE,
            lightmap_color: WimpyColorLinear::WHITE,
        }
    }
}

#[derive(Copy,Clone)]
pub enum TextureStrategy {
    Standard,
    NoLightmap,
    LightmapToDiffuse,
}

impl Pipeline3DPass<'_,'_> {
    pub fn set_transform(&mut self,transform: TransformUniform) {
        let uniform_buffer_range = self.context.pipelines
            .get_shared_mut()
            .get_uniform_buffer()
            .push(transform);

        let dynamic_offset = uniform_buffer_range.start * UNIFORM_BUFFER_ALIGNMENT;

        self.render_pass.set_bind_group(
            UNIFORM_BIND_GROUP_INDEX,
            self.context.get_shared().get_uniform_bind_group(),
            &[dynamic_offset as u32]
        );
    }

    pub fn draw(
        &mut self,
        model_data: &ModelData,
        diffuse_sampler: SamplerMode,
        texture_strategy: TextureStrategy,
        draw_data: &[DrawData3D]
    ) {

        let Some(mesh_reference) = model_data.render else {
            log::warn!("Model data's 'render' value is 'None'. Is this intentional?");
            return;
        };

        if !self.has_transform_bind {
            self.set_transform(TransformUniform::default());
        }

        if let Err(()) = self.set_mesh_textures(&TextureDrawData {
            diffuse: model_data.diffuse,
            lightmap: model_data.lightmap,
            diffuse_sampler,
            strategy: texture_strategy
        }) {
            return;
        }
        let indices = Range {
            start: mesh_reference.index_start,
            end: mesh_reference.index_end
        };
        let instances = self.context.get_3d_pipeline_mut().instance_buffer.push_set(draw_data.iter().map(Into::into));
        self.render_pass.draw_indexed(indices,mesh_reference.base_vertex,Range {
            start: instances.start as u32,
            end: instances.end as u32,
        });
    }

    fn set_mesh_textures(&mut self,texture_data: &TextureDrawData) -> Result<(),()> {

        let m = self.context.textures.missing;
        let w = self.context.textures.opaque_white;

        let (diffuse,lightmap) = match (
            texture_data.diffuse,
            texture_data.lightmap,
            texture_data.strategy
        ) {
            (None, None, _) =>                                          (m, w),

            (None, Some(l),     TextureStrategy::Standard) =>           (m, l),
            (Some(d), None,     TextureStrategy::Standard) =>           (d, w),
            (Some(d), Some(l),  TextureStrategy::Standard) =>           (d, l),

            (Some(d), _,        TextureStrategy::NoLightmap) =>         (d, w),
            (None, _,           TextureStrategy::NoLightmap) =>         (m, w),

            (_, Some(l),        TextureStrategy::LightmapToDiffuse) =>  (l, w),
            (_, None,           TextureStrategy::LightmapToDiffuse) =>  (m, w),
        };

        self.context.set_texture_bind_group(
            TEXTURE_BIND_GROUP_INDEX,
            &mut self.render_pass,
            &BindGroupCacheIdentity::DualChannel {
            ch_0: BindGroupChannelConfig {
                mode: texture_data.diffuse_sampler,
                texture: match self.context.frame_cache.get(diffuse.get_ref()) {
                    Ok(value) => value,
                    Err(error) => {
                        log::error!("Could not resolve diffuse texture frame to a texture view: {:?}",error);
                        return Err(())
                    },
                },
            },
            ch_1: BindGroupChannelConfig {
                mode: SamplerMode::LinearClamp,
                texture: match self.context.frame_cache.get(lightmap.get_ref()) {
                    Ok(value) => value,
                    Err(error) => {
                        log::error!("Could not resolve lightmap texture frame to a texture view: {:?}",error);
                        return Err(())
                    },
                }
            }
        });

        return Ok(())
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
    pub diffuse_color: [f32;4],
    pub lightmap_color: [f32;4]
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
        ATTR::DIFFUSE_COLOR => Float32x4,
        ATTR::LIGHTMAP_COLOR => Float32x4,
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
            diffuse_color: value.diffuse_color.into(),
            lightmap_color: value.lightmap_color.into(),
        }
    }
}

impl From<DrawData3D> for ModelInstance {
    fn from(value: DrawData3D) -> Self {
        ModelInstance::from(&value)
    }
}
