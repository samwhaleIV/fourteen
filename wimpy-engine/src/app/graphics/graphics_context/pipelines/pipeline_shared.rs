use super::*;

pub struct SharedPipeline {
    texture_layout: BindGroupLayout,
    uniform_layout: BindGroupLayout,
    uniform_bind_group: BindGroup,
    uniform_buffer: DoubleBuffer<TransformUniform>
}

// Not really a render pipeline. What're you going to do about it? Cry?

impl SharedPipeline {

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

    pub fn write_uniform_buffer(&mut self,queue: &Queue) {
        self.uniform_buffer.write_out_with_padding(queue,UNIFORM_BUFFER_ALIGNMENT);
    }

    pub fn reset_uniform_buffer(&mut self) {
        self.uniform_buffer.reset();
    }

    pub fn get_uniform_buffer(&mut self) -> &mut DoubleBuffer<TransformUniform> {
        return &mut self.uniform_buffer;
    }

    pub fn get_texture_layout(&self) -> &BindGroupLayout {
        return &self.texture_layout;
    }

    pub fn get_uniform_layout(&self) -> &BindGroupLayout {
        return &self.uniform_layout;
    }

    pub fn get_uniform_bind_group(&self) -> &BindGroup {
        return &self.uniform_bind_group;
    }
}
