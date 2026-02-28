use glam::Mat4;
use wgpu::*;
use bytemuck::{Pod,Zeroable};
use crate::UWimpyPoint;
use crate::app::graphics::{*,constants::*};

use super::pipeline_2d::*;
use super::pipeline_3d::*;
use super::text_pipeline::*;
use super::lines_pipeline::*;

pub struct UniquePipelines {
    pub pipeline_2d: Pipeline2D,
    pub pipeline_3d: Pipeline3D,
    pub text_pipeline: TextPipeline,
    pub lines_pipeline: LinesPipeline
}

pub struct RenderPipelines {
    pipelines_unique: UniquePipelines,
    pipeline_shared: SharedPipeline,
}

pub trait PipelineController {
    fn write_dynamic_buffers(&mut self,queue: &Queue);
    fn reset_pipeline_state(&mut self);
}

impl RenderPipelines {
    pub fn create<TConfig>(
        graphics_provider: &GraphicsProvider,
        texture_bind_group_layout: &BindGroupLayout
    ) -> Self
    where
        TConfig: GraphicsContextConfig
    {
        let pipeline_shared = SharedPipeline::create::<TConfig>(graphics_provider);
        let uniform_bind_group_layout = pipeline_shared.get_uniform_layout();

        let pipeline_2d = Pipeline2D::create::<TConfig>(
            graphics_provider,
            texture_bind_group_layout,
            uniform_bind_group_layout
        );

        let pipeline_3d = Pipeline3D::create::<TConfig>(
            graphics_provider,
            texture_bind_group_layout,
            uniform_bind_group_layout
        );

        let text_pipeline = TextPipeline::create::<TConfig>(
            graphics_provider,
            texture_bind_group_layout,
            uniform_bind_group_layout
        );

        let lines_pipeline = LinesPipeline::create::<TConfig>(
            graphics_provider,
            texture_bind_group_layout,
            uniform_bind_group_layout
        );

        return Self {
            pipelines_unique: UniquePipelines {
                pipeline_2d,
                pipeline_3d,
                text_pipeline,
                lines_pipeline,
            },
            pipeline_shared,
        }
    }

    pub fn write_pipeline_buffers(&mut self,queue: &Queue) {
        // Investigate: only finalize the pipelines that were used during this output builder's life (or let the pipelines no-op on their own?)
        self.pipelines_unique.pipeline_2d.write_dynamic_buffers(queue);
        self.pipelines_unique.pipeline_3d.write_dynamic_buffers(queue);
        self.pipelines_unique.text_pipeline.write_dynamic_buffers(queue);
        self.pipelines_unique.lines_pipeline.write_dynamic_buffers(queue);
        // We always write the shared buffers
        self.pipeline_shared.write_uniform_buffer(queue);
    }

    pub fn reset_pipeline_states(&mut self) {
        self.pipelines_unique.pipeline_2d.reset_pipeline_state();
        self.pipelines_unique.pipeline_3d.reset_pipeline_state();
        self.pipelines_unique.text_pipeline.reset_pipeline_state();
        self.pipelines_unique.lines_pipeline.reset_pipeline_state();
        
        self.pipeline_shared.reset_uniform_buffer();
    }

    pub fn get_shared(&self) -> &SharedPipeline {
        return &self.pipeline_shared;
    }

    pub fn get_shared_mut(&mut self) -> &mut SharedPipeline {
        return &mut self.pipeline_shared;
    }

    pub fn get_unique(&self) -> &UniquePipelines {
        return &self.pipelines_unique;
    }

    pub fn get_unique_mut(&mut self) -> &mut UniquePipelines {
        return &mut self.pipelines_unique;
    }
}

#[repr(C)]
#[derive(Debug,Copy,Clone,Pod,Zeroable)]
pub struct TransformUniform {
    pub view_projection: Mat4
}

impl Default for TransformUniform {
    fn default() -> Self {
        Self {
            view_projection: glam::Mat4::IDENTITY
        }
    }
}

impl TransformUniform {
    pub fn create_ortho(size: UWimpyPoint) -> Self {
        let view_projection = glam::Mat4::orthographic_rh(
            0.0,
            size.x as f32,
            size.y as f32,
            0.0,
            0.0,
            1.0,
        );
        return Self {
            view_projection
        };
    }
}

pub struct PipelineCreator<'a> {
    pub graphics_provider: &'a GraphicsProvider,
    pub render_pipeline_layout: &'a PipelineLayout,
    pub shader: &'a ShaderModule,
    pub vertex_buffer_layout: &'a [VertexBufferLayout<'a>],
    pub primitive_state: &'a PrimitiveState,
    pub label: &'static str,
}

impl PipelineCreator<'_> {
    fn create_pipeline(
        &self,
        texture_format: TextureFormat
    ) -> RenderPipeline {
        let pipeline = self.graphics_provider.get_device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(self.label),
            layout: Some(self.render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: self.shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: self.vertex_buffer_layout
            },
            fragment: Some(wgpu::FragmentState {
                module: self.shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format, // Match to the texture view format, not the underlying storage format of the texture/surface
                    blend: Some(wgpu::BlendState {
                        color: BlendComponent {
                            src_factor: BlendFactor::SrcAlpha,
                            dst_factor: BlendFactor::OneMinusSrcAlpha,
                            operation: BlendOperation::Add,
                        },
                        alpha: BlendComponent {
                            src_factor: BlendFactor::One,
                            dst_factor: BlendFactor::OneMinusSrcAlpha,
                            operation: BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })]
            }),
            primitive: self.primitive_state.clone(),
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
        return pipeline;
    }

    pub fn create_pipeline_set(&self) -> PipelineSet {
        PipelineSet {
            internal_target_pipeline: self.create_pipeline(
                INTERNAL_RENDER_TARGET_FORMAT
            ),
            output_surface_pipeline: self.create_pipeline(
                self.graphics_provider.get_output_view_format()
            ),
        }
    }
}

#[derive(Copy,Clone)]
pub enum PipelineVariantKey {
    InternalTarget,
    OutputSurface
}

pub struct PipelineSet {
    internal_target_pipeline: RenderPipeline,
    output_surface_pipeline: RenderPipeline
}

impl PipelineSet {
    pub fn select(&self,key: PipelineVariantKey) -> &RenderPipeline {
        use PipelineVariantKey::*;
        match key {
            OutputSurface => &self.output_surface_pipeline,
            InternalTarget => &self.internal_target_pipeline,
        }
    }
}

pub struct SharedPipeline {
    uniform_layout: BindGroupLayout,
    uniform_bind_group: BindGroup,
    uniform_buffer: DoubleBuffer<TransformUniform>,
    current_uniform_bind: Option<u32>
}

#[derive(Copy,Clone)]
pub struct UniformReference {
    value: u32
}

// Not really a render pipeline. What're you going to do about it? Cry?

impl SharedPipeline {

    pub fn create<TConfig>(graphics_provider: &GraphicsProvider) -> Self
    where
        TConfig: GraphicsContextConfig
    {

        let device = graphics_provider.get_device();

        let chunk_size = std::num::NonZero::new(UNIFORM_BUFFER_ALIGNMENT as BufferAddress).expect("valid chunk size");

        let uniform_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: UNIFORM_BIND_GROUP_ENTRY_INDEX,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: Some(chunk_size),
                    },
                    count: None,
                }
            ],
            label: Some("Uniform Bind Group Layout"),
        });

        let uniform_buffer = DoubleBuffer::new(device.create_buffer(&BufferDescriptor {
            label: Some("Uniform Buffer"),
            //See: https://docs.rs/wgpu-types/27.0.1/wgpu_types/struct.Limits.html#structfield.min_storage_buffer_offset_alignment
            size: TConfig::UNIFORM_BUFFER_SIZE as BufferAddress,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false
        }));

        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &uniform_layout,
            entries: &[BindGroupEntry {
                binding: UNIFORM_BIND_GROUP_ENTRY_INDEX,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: &uniform_buffer.get_output_buffer(),
                    offset: 0,
                    size: Some(chunk_size),
                }),
            }],
            label: Some("Uniform Bind Group"),
        });

        return Self {
            uniform_layout,
            uniform_bind_group,
            uniform_buffer,
            current_uniform_bind: None,
        }
    }

    pub fn write_uniform_buffer(&mut self,queue: &Queue) {
        self.uniform_buffer.write_out_with_padding(queue,UNIFORM_BUFFER_ALIGNMENT);
    }

    pub fn reset_uniform_buffer(&mut self) {
        self.uniform_buffer.reset();
        self.current_uniform_bind = None;
    }

    pub fn get_uniform_buffer(&mut self) -> &mut DoubleBuffer<TransformUniform> {
        return &mut self.uniform_buffer;
    }

    pub fn get_uniform_layout(&self) -> &BindGroupLayout {
        return &self.uniform_layout;
    }

    pub fn get_uniform_bind_group(&self) -> &BindGroup {
        return &self.uniform_bind_group;
    }

    pub fn create_uniform_ortho(&mut self,size: UWimpyPoint) -> UniformReference {
        let transform = TransformUniform::create_ortho(size);
        let uniform_buffer_range = self.get_uniform_buffer().push(transform);
        UniformReference {
            value: (uniform_buffer_range.start * UNIFORM_BUFFER_ALIGNMENT) as u32,
        }
    }

    pub fn create_uniform(&mut self,view_projection: Mat4) -> UniformReference {
        let transform = TransformUniform {
            view_projection,
        };
        let uniform_buffer_range = self.get_uniform_buffer().push(transform);
        UniformReference {
            value: (uniform_buffer_range.start * UNIFORM_BUFFER_ALIGNMENT) as u32,
        }
    }

    pub fn set_uniform(&mut self,render_pass: &mut RenderPass,uniform_reference: UniformReference) {
        let new_value = uniform_reference.value;
        if let Some(cur_value) = self.current_uniform_bind && cur_value == new_value {
            return;
        }
        render_pass.set_bind_group(
            UNIFORM_BIND_GROUP_INDEX,
            self.get_uniform_bind_group(),
            &[new_value]
        );
        self.current_uniform_bind = Some(new_value);
    }
}
