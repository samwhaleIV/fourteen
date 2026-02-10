use super::prelude::*;

mod pipeline_2d;
mod pipeline_3d;

pub use pipeline_2d::*;
pub use pipeline_3d::*;

pub struct RenderPipelines {
    pub pipeline_2d: Pipeline2D,
    pub pipeline_3d: Pipeline3D,
    pub shared: SharedPipelineSet,
}

pub trait RenderPassController {

    fn begin(
        &mut self,
        render_pass: &mut RenderPass,
        shared_pipeline: &mut SharedPipelineSet,
        uniform: CameraUniform
    );

    fn select_and_begin(
        render_pass: &mut RenderPass,
        render_pipelines: &mut RenderPipelines,
        uniform: CameraUniform
    );

    fn write_buffers(&mut self,queue: &Queue);

    fn reset_buffers(&mut self);
}

impl RenderPipelines {
    pub fn create<TConfig>(graphics_provider: &GraphicsProvider) -> Self
    where
        TConfig: GraphicsContextConfig
    {
        let shared_pipeline_set = SharedPipelineSet::create::<TConfig>(graphics_provider);
        let pipeline_2d = Pipeline2D::create::<TConfig>(
            graphics_provider,
            &shared_pipeline_set
        );
        let pipeline_3d = Pipeline3D::create::<TConfig>(
            graphics_provider,
            &shared_pipeline_set
        );
        return Self {
            pipeline_2d,
            pipeline_3d,
            shared: shared_pipeline_set,
        }
    }
    pub fn reset_buffers(&mut self) {
        self.pipeline_2d.reset_buffers();
        self.pipeline_3d.reset_buffers();
        self.shared.reset_buffers();
    }
}

pub struct SharedPipelineSet {
    pub texture_layout: BindGroupLayout,
    pub uniform_layout: BindGroupLayout,
    pub uniform_bind_group: BindGroup,
    pub uniform_buffer: DoubleBuffer<CameraUniform>
}

impl SharedPipelineSet {

    pub fn create<TConfig>(graphics_provider: &GraphicsProvider) -> Self
    where
        TConfig: GraphicsContextConfig
    {

        let device = graphics_provider.get_device();

        let texture_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Texture Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: DIFFUSE_TEXTURE_BIND_GROUP_ENTRY_INDEX,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false, /* Must remain false to use STORAGE_BINDING texture usage */
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float {
                            filterable: true
                        },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: DIFFUSE_SAMPLER_BIND_GROUP_ENTRY_INDEX,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: LIGHTMAP_TEXTURE_BIND_GROUP_ENTRY_INDEX,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float {
                            filterable: true
                        },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: LIGHTMAP_SAMPLER_BIND_GROUP_ENTRY_INDEX,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ]
        });

        let uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: CAMERA_UNIFORM_BIND_GROUP_ENTRY_INDEX,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("Uniform Bind Group Layout"),
        });

        let uniform_buffer = DoubleBuffer::new(device.create_buffer(&BufferDescriptor {
            label: Some("Uniform Buffer"),
            //See: https://docs.rs/wgpu-types/27.0.1/wgpu_types/struct.Limits.html#structfield.min_storage_buffer_offset_alignment
            size: TConfig::UNIFORM_BUFFER_SIZE as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false
        }));

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: CAMERA_UNIFORM_BIND_GROUP_ENTRY_INDEX,
                resource: uniform_buffer.get_output_buffer().as_entire_binding(),
            }],
            label: Some("Uniform Bind Group"),
        });

        return Self {
            texture_layout,
            uniform_layout,
            uniform_bind_group,
            uniform_buffer,
        }
    }

    pub fn write_buffers(&mut self,queue: &Queue) {
        self.uniform_buffer.write_out_with_padding(queue,UNIFORM_BUFFER_ALIGNMENT);
    }

    pub fn reset_buffers(&mut self) {
        self.uniform_buffer.reset();
    }
}

#[repr(C)]
#[derive(Debug,Copy,Clone,Pod,Zeroable)]
pub struct CameraUniform {
    pub view_projection: [[f32;4];4]
}

impl CameraUniform {
    pub fn placeholder() -> Self {
        return Self {
            view_projection: Default::default()
        }
    }
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
            view_projection
        };
    }
}
