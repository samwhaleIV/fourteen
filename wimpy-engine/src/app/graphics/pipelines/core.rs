use glam::Mat4;
use wgpu::*;
use bytemuck::{Pod,Zeroable};

use super::*;
use crate::UWimpyPoint;
use crate::app::graphics::{*,GraphicsConfig, textures::TextureManager};

pub struct RenderPipelines {
    pub pipeline_2d:    Pipeline2D,
    pub pipeline_3d:    Pipeline3D,
    pub text:           TextPipeline,
    pub lines:          LinesPipeline,
    pub core:           PipelineCore
}

pub trait PipelineFlush {
    /// Write out and clear buffers that are built frame by frame, such as instance buffers
    /// 
    /// Not intended for static buffers such as vertex or index buffers
    fn flush(&mut self,queue: &Queue);
}

/* GraphicsContext isn't fully formed during pipeline creation; but pipelines are a part of the graphics context */
pub struct PipelineCreationContext<'a> {
    pub graphics_provider:  &'a GraphicsProvider,
    pub core:               &'a PipelineCore,
    pub texture_manager:    &'a mut TextureManager,
    pub mesh_cache:         &'a mut MeshCache,
}

impl RenderPipelines {
    pub fn create<TConfig>(
        graphics_provider:  &GraphicsProvider,
        texture_manager:    &mut TextureManager,
        mesh_cache:         &mut MeshCache,
        pipeline_core:      PipelineCore,
    ) -> Self
    where
        TConfig: GraphicsConfig
    {
        let context = PipelineCreationContext {
            graphics_provider,
            core: &pipeline_core,
            texture_manager,
            mesh_cache
        };

        let pipeline_2d = Pipeline2D::create::<TConfig>(&context);
        let pipeline_3d = Pipeline3D::create::<TConfig>(&context);
        let text_pipeline = TextPipeline::create::<TConfig>(&context);
        let lines_pipeline = LinesPipeline::create::<TConfig>(&context);

        return Self {
            pipeline_2d,
            pipeline_3d,
            text: text_pipeline,
            lines: lines_pipeline,
            core: pipeline_core,
        }
    }

    pub fn flush(&mut self,queue: &Queue) {

        self.pipeline_2d.flush(queue);
        self.pipeline_3d.flush(queue);
        self.text.flush(queue);
        self.lines.flush(queue);

        let uniform_buffer = &mut self.core.uniform_buffer;
        uniform_buffer.write_out_with_padding(queue,constants::UNIFORM_BUFFER_ALIGNMENT);
        uniform_buffer.reset();
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

enum DepthStencilMode {
    None,
    Standard
}

impl PipelineCreator<'_> {
    fn create_pipeline(
        &self,
        texture_format: TextureFormat,
        depth_stencil_mode: DepthStencilMode
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
            depth_stencil: match depth_stencil_mode {
                DepthStencilMode::None => None,
                DepthStencilMode::Standard => Some(DepthStencilState {
                    format:                 constants::DEPTH_STENCIL_TEXTURE_FORMAT,
                    depth_write_enabled:    true,
                    depth_compare:          CompareFunction::Less,
                    stencil:                StencilState::default(),
                    bias:                   DepthBiasState::default(),
                })
            },
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None
        });
        return pipeline;
    }

    pub fn create_pipeline_set(&self) -> PipelineVariants {
        PipelineVariants {
            internal_target_pipeline: self.create_pipeline(
                constants::INTERNAL_RENDER_TARGET_FORMAT,
                DepthStencilMode::None
            ),
            output_surface_pipeline: self.create_pipeline(
                self.graphics_provider.get_output_view_format(),
                DepthStencilMode::None
            ),
            internal_target_pipeline_with_depth: self.create_pipeline(
                constants::INTERNAL_RENDER_TARGET_FORMAT,
                DepthStencilMode::Standard
            ),
            output_surface_pipeline_with_depth: self.create_pipeline(
                self.graphics_provider.get_output_view_format(),
                DepthStencilMode::Standard
            ),
        }
    }
}

#[derive(Copy,Clone)]
pub enum PipelineVariantKey {
    RenderTarget,
    OutputSurface,
    InternalTargetWithDepth,
    OutputSurfaceWithDepth
}

/// Provides variadic pipeline selection for handling format mismatches between internal render targets and the ultimate output surface
pub struct PipelineVariants {
    internal_target_pipeline: RenderPipeline,
    output_surface_pipeline: RenderPipeline,
    internal_target_pipeline_with_depth: RenderPipeline,
    output_surface_pipeline_with_depth: RenderPipeline
}

impl PipelineVariants {
    pub fn select(&self,key: PipelineVariantKey) -> &RenderPipeline {
        use PipelineVariantKey::*;
        match key {
            OutputSurface => {
                &self.output_surface_pipeline
            },
            RenderTarget => {
                &self.internal_target_pipeline
            },
            InternalTargetWithDepth => {
                &self.internal_target_pipeline_with_depth
            },
            OutputSurfaceWithDepth => {
                &self.output_surface_pipeline_with_depth
            },
        }
    }
}

/// A core set of values (such as bind group layouts) used across various independent pipelines
pub struct PipelineCore {
    pub uniform_layout:     BindGroupLayout,
    pub texture_layout:     BindGroupLayout,
    pub uniform_bind_group: BindGroup,
    pub uniform_buffer:     DoubleBuffer<TransformUniform>,
}

#[derive(Copy,Clone)]
pub struct UniformReference {
    value: u32
}

// Not really a render pipeline. What're you going to do about it? Cry?

impl PipelineCore {

    pub fn create<TConfig>(graphics_provider: &GraphicsProvider) -> Self
    where
        TConfig: GraphicsConfig
    {

        let device = graphics_provider.get_device();

        let chunk_size = std::num::NonZero::new(constants::UNIFORM_BUFFER_ALIGNMENT as BufferAddress).expect("valid chunk size");

        let uniform_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: constants::UNIFORM_BIND_GROUP_ENTRY_INDEX,
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


        let texture_layout = graphics_provider.get_device().create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Texture Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: constants::CH0_TEXTURE_INDEX,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false, /* Must remain false to use STORAGE_BINDING texture usage */
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float {
                            filterable: true
                        },
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: constants::CH0_SAMPLER_INDEX,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: constants::CH1_TEXTURE_INDEX,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float {
                            filterable: true
                        },
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: constants::CH1_SAMPLER_INDEX,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ]
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
                binding: constants::UNIFORM_BIND_GROUP_ENTRY_INDEX,
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
            texture_layout,
            uniform_bind_group,
            uniform_buffer,
        }
    }

    pub fn create_uniform_ortho(&mut self,size: UWimpyPoint) -> UniformReference {
        let transform = TransformUniform::create_ortho(size);
        let uniform_buffer_range = self.uniform_buffer.push(transform);
        UniformReference {
            value: (uniform_buffer_range.start * constants::UNIFORM_BUFFER_ALIGNMENT) as u32,
        }
    }

    pub fn create_uniform(&mut self,view_projection: Mat4) -> UniformReference {
        let transform = TransformUniform { view_projection };
        let uniform_buffer_range = self.uniform_buffer.push(transform);
        UniformReference {
            value: (uniform_buffer_range.start * constants::UNIFORM_BUFFER_ALIGNMENT) as u32,
        }
    }

    pub fn bind_uniform<const BIND_GROUP_INDEX: u32>(&self,render_pass: &mut RenderPass,uniform_reference: UniformReference) {
        render_pass.set_bind_group(
            BIND_GROUP_INDEX,
            &self.uniform_bind_group,
            &[uniform_reference.value]
        );
    }
}
