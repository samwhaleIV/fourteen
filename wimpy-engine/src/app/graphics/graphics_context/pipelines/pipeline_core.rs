use super::*;

pub struct UniquePipelines {
    pub pipeline_2d: Pipeline2D,
    pub pipeline_3d: Pipeline3D,
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

        return Self {
            pipelines_unique: UniquePipelines {
                pipeline_2d,
                pipeline_3d
            },
            pipeline_shared,
        }
    }

    pub fn write_pipeline_buffers(&mut self,queue: &Queue) {
        // Investigate: only finalize the pipelines that were used during this output builder's life (or let the pipelines no-op on their own?)
        self.pipelines_unique.pipeline_2d.write_dynamic_buffers(queue);
        self.pipelines_unique.pipeline_3d.write_dynamic_buffers(queue);

        // We always write the shared buffers
        self.pipeline_shared.write_uniform_buffer(queue);
    }

    pub fn reset_pipeline_states(&mut self) {
        self.pipelines_unique.pipeline_2d.reset_pipeline_state();
        self.pipelines_unique.pipeline_3d.reset_pipeline_state();
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
    pub value: [[f32;4];4]
}

impl Default for TransformUniform {
    fn default() -> Self {
        Self {
            value: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ]
        }
    }
}

impl TransformUniform {
    pub fn create_ortho(size: (u32,u32)) -> Self {
        let (width,height) = size;

        let view_projection = cgmath::ortho(
            0.0, //Left
            width as f32, //Right
            height as f32, //Bottom
            0.0, //Top
            -1.0, //Near
            1.0, //Far
        ).into();

        return Self {
            value: view_projection
        };
    }
}
