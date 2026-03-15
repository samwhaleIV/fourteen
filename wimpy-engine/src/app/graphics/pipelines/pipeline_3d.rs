use glam::Mat4;
use wgpu::*;
use std::num::NonZero;
use bytemuck::{Pod,Zeroable};
use crate::app::{graphics::*};
use super::core::*;
use constants::pipeline_3d::*;

pub struct Pipeline3D {
    diffuse_atlas: VirtualTextureAtlas,
    lightmap_atlas: VirtualTextureAtlas,
    variants: PipelineVariants,
    storage_bind_group: BindGroup,
    external_instance_buffer: Buffer,
    instance_buckets: InstanceBucketSet
}

struct InstanceBucket {
    largest: u32,
    buffer: Vec<MeshInstance>
}

impl Default for InstanceBucket {
    fn default() -> Self {
        Self {
            largest: u32::MIN,
            buffer: Vec::with_capacity(INSTANCE_BUFFER_BUCKET_START_SIZE),
        }
    }
}

impl InstanceBucket {
    fn clear(&mut self) {
        self.largest = u32::MIN;
        self.buffer.clear();
    }
}

#[derive(Default)]
struct InstanceBucketSet {
    /// Amount of items across all buckets
    instance_count: usize,
    buckets: [InstanceBucket;INSTANCE_BUFFER_BUCKET_COUNT as usize]
}

impl InstanceBucketSet {
    fn flush(&mut self,buffer: &Buffer,queue: &Queue) {
        if
            let Some(size) = NonZero::new((self.instance_count * size_of::<MeshInstance>()) as BufferAddress) &&
            let Some(mut buffer_view) = queue.write_buffer_with(buffer,0,size)
        {
            let mut offset: usize = 0;
            for bucket in self.buckets.iter() {
                let bytes = bytemuck::cast_slice(&bucket.buffer);
                buffer_view[offset..offset + bytes.len()].copy_from_slice(bytes);
                offset += bytes.len();
            }
        }
        for bucket in self.buckets.iter_mut() {
            bucket.clear();
        }
        self.instance_count = 0;
    }

    fn push(&mut self,instance: MeshInstance) {
        let Some(log2) = instance.index_count.checked_ilog2() else {
            return;
        };
        let bucket_index = log2.saturating_sub(SMALLEST_BUCKET_LIMIT_POW_OF_2).min(INSTANCE_BUFFER_BUCKET_COUNT - 1);
        let bucket = &mut self.buckets[bucket_index as usize];
        bucket.largest = bucket.largest.max(instance.index_count);
        bucket.buffer.push(instance);
        self.instance_count += 1;
    }
}

// Bind group indices
const TEXTURE_BG: u32 = 0;
const UNIFORM_BG: u32 = 1;
const STORAGE_BG: u32 = 2;

// Bind group entry indices
const STORAGE_BG_VERTICES: u32 = 0;
const STORAGE_BG_INDICES: u32 = 1;
const STORAGE_BG_INSTANCES: u32 = 2;

impl Pipeline3D {

    pub fn create<TConfig>(
        graphics_provider: &GraphicsProvider,
        texture_layout: &BindGroupLayout,
        uniform_layout: &BindGroupLayout,
        texture_id_generator: &mut TextureIdentityGenerator,
        mesh_cache: &MeshCache
    ) -> Self
    where
        TConfig: GraphicsContextConfig
    {
        let device = graphics_provider.get_device();

        let shader = &device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Pipeline 3D Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/pipeline3D.wgsl").into())
        });

        let storage_layout_entry = |binding| {
            BindGroupLayoutEntry {
                binding,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage {
                        read_only: true
                    },
                    has_dynamic_offset: false,
                    min_binding_size: None
                },
                count: None,
            }
        };

        let storage_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Pipeline 3D Storage Bind Group Layout"),
            entries: &[
                // Vertex Buffer
                storage_layout_entry(STORAGE_BG_VERTICES),
                // Index Buffer
                storage_layout_entry(STORAGE_BG_INDICES),
                // Instance Buffer
                storage_layout_entry(STORAGE_BG_INSTANCES)
            ]
        });

        let render_pipeline_layout = &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline 3D Render Layout"),
            bind_group_layouts: &[
                texture_layout,
                uniform_layout,
                &storage_bind_group_layout
            ],
            push_constant_ranges: &[]
        });

        let pipeline_creator = PipelineCreator {
            graphics_provider,
            render_pipeline_layout,
            shader,
            vertex_buffer_layout: &[],
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

        let instance_buffer = device.create_buffer(&BufferDescriptor{
            label: Some("Pipeline 3D Instance Buffer"),
            size: TConfig::INSTANCE_BUFFER_SIZE_3D as BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let diffuse_atlas = VirtualTextureAtlas::create(
            graphics_provider,
            texture_id_generator.next(),
            &VirtualTextureAtlasConfig {
                slot_size: ATLAS_SLOT_SIZE_DIFFUSE,
                slot_length: ATLAS_SLOT_LENGTH_DIFFUSE,
            }
        );

        let lightmap_atlas = VirtualTextureAtlas::create(
            graphics_provider,
            texture_id_generator.next(),
            &VirtualTextureAtlasConfig {
                slot_size: ATLAS_SLOT_SIZE_LIGHTMAP,
                slot_length: ATLAS_SLOT_LENGTH_LIGHTMAP,
            }
        );

        let storage_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Pipeline 3D Storage Bind Group"),
            layout: &storage_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: STORAGE_BG_VERTICES,
                    resource: mesh_cache.get_vertex_buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: STORAGE_BG_INDICES,
                    resource: mesh_cache.get_index_buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: STORAGE_BG_INSTANCES,
                    resource: instance_buffer.as_entire_binding(),
                },
            ]
        });

        let instance_buffer_buckets: InstanceBucketSet = Default::default();

        return Self {
            diffuse_atlas,
            lightmap_atlas,
            variants: pipelines,
            external_instance_buffer: instance_buffer,
            storage_bind_group,
            instance_buckets: instance_buffer_buckets,
        }
    }

    pub fn flush_encoder(&mut self,encoder: &mut CommandEncoder) {
        self.diffuse_atlas.flush(encoder);
        self.lightmap_atlas.flush(encoder);
    }

    pub fn batch<I>(context: &mut GraphicsContext,texture_strategy: TextureStrategy,draw_data: I)
    where
        I: IntoIterator<Item = DrawData3D>,
    {
        let pipeline_3d = &mut context.pipelines.pipeline_3d;

        for draw_data in draw_data.into_iter() {

            let meshlets = context.mesh_cache.get_textured_mesh_ref(draw_data.mesh);

            let transform_0 = draw_data.transform.x_axis.into();
            let transform_1 = draw_data.transform.y_axis.into();
            let transform_2 = draw_data.transform.z_axis.into();
            let transform_3 = draw_data.transform.w_axis.into();

            for meshlet in meshlets {
                let (diffuse,lightmap) = match texture_strategy {
                    TextureStrategy::Standard => (meshlet.diffuse,meshlet.lightmap),
                    TextureStrategy::LightmapToDiffuse => (
                        meshlet.lightmap,
                        context.engine_textures.opaque_white
                    ),
                    TextureStrategy::NoLightmap => (
                        meshlet.diffuse,
                        context.engine_textures.opaque_white
                    )
                };

                let uv_diffuse = pipeline_3d.diffuse_atlas.set_texture(
                    &context.frame_cache,
                    diffuse.get_ref()
                );

                let uv_lightmap = pipeline_3d.lightmap_atlas.set_texture(
                    &context.frame_cache,
                    lightmap.get_ref()
                );

                let range = &meshlet.range;

                pipeline_3d.instance_buckets.push(MeshInstance {
                    uv_diffuse: uv_diffuse.into(),
                    uv_lightmap: uv_lightmap.into(),

                    transform_0,
                    transform_1,
                    transform_2,
                    transform_3,

                    base_vertex: range.base_vertex,
                    index_start: range.index_start,
                    index_count: range.index_count,

                    _padding: 0.0
                });
            }
        }
    }
}

impl PipelineFlush for Pipeline3D {
    fn flush(&mut self,queue: &Queue) {
        self.instance_buckets.flush(&self.external_instance_buffer,queue);
    }
}

pub struct Pipeline3DPass<'pass,'encoder> {
    render_pass: &'pass mut RenderPass<'encoder>,
    context: &'pass mut GraphicsContext,
    variant_key: PipelineVariantKey,
    uniform_reference: UniformReference
}

impl<'pass,'context> PipelinePass<'pass,'context> for Pipeline3DPass<'pass,'context> {
    fn create(
        render_pass: &'pass mut RenderPass<'context>,
        context: &'pass mut GraphicsContext,
        variant_key: PipelineVariantKey,
        uniform_reference: UniformReference,
    ) -> Self {
        Self {
            render_pass,
            context,
            variant_key,
            uniform_reference,
        }
    }
}

pub struct DrawData3D {
    pub transform: Mat4,
    pub mesh: TexturedMeshReference
}

#[derive(Copy,Clone)]
pub enum TextureStrategy {
    Standard,
    NoLightmap,
    LightmapToDiffuse,
}

impl Pipeline3DPass<'_,'_> {
    pub fn submit(&mut self,diffuse_sampler: SamplerMode) {
        let pipeline = &self.context.pipelines.pipeline_3d;

        if pipeline.instance_buckets.instance_count <= 0 {
            return;
        }

        self.render_pass.set_pipeline(pipeline.variants.select(self.variant_key));

        self.context.pipelines.shared.bind_uniform::<UNIFORM_BG>(self.render_pass,self.uniform_reference);
        self.render_pass.set_bind_group(STORAGE_BG,&pipeline.storage_bind_group,&[]);

        let bind_group = self.context.bind_group_cache.get(
            self.context.graphics_provider.get_device(),
            &BindGroupCacheIdentity::DualChannel {
                ch_0: BindGroupChannelConfig {
                    mode: diffuse_sampler,
                    texture: &pipeline.diffuse_atlas.get_texture_container(),
                },
                ch_1: BindGroupChannelConfig {
                    mode: SamplerMode::LinearClamp,
                    texture: &pipeline.lightmap_atlas.get_texture_container(),
                }
            }
        );
        self.render_pass.set_bind_group(TEXTURE_BG,bind_group,&[]);

        let buckets = &pipeline.instance_buckets.buckets;
        let mut offset: u32 = 0;

        // Reasonably sized meshes, vertex discards offset greatly by draw call reduction
        for bucket in &buckets[..buckets.len() - 1] {
            let instances = bucket.buffer.len() as u32;
            if instances <= 0 {
                continue;
            }
            self.render_pass.draw(0..bucket.largest,offset..offset + instances);
            offset += instances;
        }

        // Very large meshes, high potential for extreme vertex discards
        for instance in &buckets[buckets.len() - 1].buffer {
            self.render_pass.draw(0..instance.index_count,offset..offset + 1);
            offset += 1;
        }
    }
}

#[repr(C)]
#[derive(Copy,Clone,Debug,Default,Pod,Zeroable)]
pub struct MeshVertex {
    pub uv_diffuse: [f32;2],
    pub uv_lightmap: [f32;2],
    pub position: [f32;3],
    pub _padding: f32
}

#[repr(C)]
#[derive(Copy,Clone,Debug,Default,Pod,Zeroable)]
pub struct MeshInstance {
    pub uv_diffuse: [f32;4],
    pub uv_lightmap: [f32;4],

    pub transform_0: [f32;4],
    pub transform_1: [f32;4],
    pub transform_2: [f32;4],
    pub transform_3: [f32;4],

    pub base_vertex: u32,
    pub index_start: u32,
    pub index_count: u32,

    pub _padding: f32
}
