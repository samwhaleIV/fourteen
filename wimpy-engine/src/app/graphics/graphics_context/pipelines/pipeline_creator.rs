use super::*;

enum SrgbStrategy {
    None,
    LinearToSrgb
}

pub struct PipelineCreator<'a> {
    pub graphics_provider: &'a GraphicsProvider,
    pub render_pipeline_layout: &'a PipelineLayout,
    pub shader: &'a ShaderModule,
    pub vertex_buffer_layout: &'a [VertexBufferLayout<'a>],
    pub primitive_state: &'a PrimitiveState,
    pub label: &'static str,
}

pub struct PipelineVariants {
    pub output_surface: RenderPipeline,
    pub render_target: RenderPipeline
}

impl PipelineVariants {
    pub fn select(&self,frame: &impl MutableFrame) -> &RenderPipeline {
        return match frame.is_output_surface() {
            true => &self.output_surface,
            false => &self.render_target,
        }
    }
}

impl PipelineCreator<'_> {
    fn create_pipeline(
        &self,
        srgb_strategy: SrgbStrategy,
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
                entry_point: Some(match srgb_strategy {
                    SrgbStrategy::LinearToSrgb => "fs_to_srgb",
                    SrgbStrategy::None => "fs_no_srgb",
                }),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
    pub fn create_variants(&self) -> PipelineVariants {
        let render_target = self.create_pipeline(
            SrgbStrategy::None,
            INTERNAL_RENDER_TARGET_FORMAT,
        );

        let output_surface_format = self.graphics_provider.get_output_format();

        let output_surface = self.create_pipeline(
            match output_surface_format.is_srgb() {
                true => SrgbStrategy::None,
                false => SrgbStrategy::LinearToSrgb,
            },
            output_surface_format
        );

        return PipelineVariants {
            output_surface,
            render_target,
        }
    }
}
