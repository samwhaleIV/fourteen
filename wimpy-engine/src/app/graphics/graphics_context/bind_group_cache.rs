use super::prelude::*;

const SAMPLER_COUNT: usize = 6;

#[derive(PartialEq,Eq,Copy,Clone)]
pub enum SamplerMode {
    NearestClamp,
    NearestWrap,
    NearestWrapMirror,
    LinearClamp,
    LinearWrap,
    LinearWrapMirror
}

const BIND_GROUP_SETS: [(FilterMode,AddressMode);SAMPLER_COUNT] = [
    (FilterMode::Nearest,AddressMode::ClampToEdge),
    (FilterMode::Nearest,AddressMode::Repeat),
    (FilterMode::Nearest,AddressMode::MirrorRepeat),
    (FilterMode::Linear,AddressMode::ClampToEdge),
    (FilterMode::Linear,AddressMode::Repeat),
    (FilterMode::Linear,AddressMode::MirrorRepeat),
];

const fn get_bind_group_index(filter_mode: FilterMode,address_mode: AddressMode) -> usize {
    match (filter_mode,address_mode) {
        (FilterMode::Nearest, AddressMode::ClampToEdge) => 0,
        (FilterMode::Nearest, AddressMode::Repeat) => 1,
        (FilterMode::Nearest, AddressMode::MirrorRepeat) => 2,

        (FilterMode::Linear, AddressMode::ClampToEdge) => 3,
        (FilterMode::Linear, AddressMode::Repeat) => 4,
        (FilterMode::Linear, AddressMode::MirrorRepeat) => 5,

        (FilterMode::Nearest, AddressMode::ClampToBorder) => 0, // Mask to ClampToEdge
        (FilterMode::Linear, AddressMode::ClampToBorder) => 3, // Mask to ClampToEdge
    }
}

pub struct Samplers {
    value: [Sampler;SAMPLER_COUNT]
}

impl Samplers {
    pub fn create(device: &Device) -> Self {
        let samplers: [Sampler;SAMPLER_COUNT] = std::array::from_fn(|i|{
            let (filter,address) = BIND_GROUP_SETS[i];
            device.create_sampler(&SamplerDescriptor {
                address_mode_u: address,
                address_mode_v: address,
                address_mode_w: address,
                mag_filter: filter,
                min_filter: filter,
                mipmap_filter: filter,
                ..Default::default()
            })
        });
        return Self {
            value: samplers
        };
    }
}

// fn create_bind_groups(texture_view: &TextureView,samplers: &Samplers,device: &Device,bind_group_layout: &BindGroupLayout) -> [BindGroup;SAMPLER_COUNT] {
//     let bind_groups: [BindGroup;SAMPLER_COUNT] = std::array::from_fn(|i| device.create_bind_group(&BindGroupDescriptor {
//         label: Some("Texture Bind Group"),
//         layout: bind_group_layout,
//         entries: &[
//             BindGroupEntry {
//                 binding: DIFFUSE_TEXTURE_BIND_GROUP_ENTRY_INDEX,
//                 resource: BindingResource::TextureView(&texture_view),
//             },
//             BindGroupEntry {
//                 binding: DIFFUSE_SAMPLER_BIND_GROUP_ENTRY_INDEX,
//                 resource: BindingResource::Sampler(&samplers.value[i]),
//             }
//         ],
//     }));
//     return bind_groups;
// }


pub struct BindGroupCache {
    samplers: Samplers,
    bind_group_layout: BindGroupLayout,
}

pub struct BindGroupCacheChannel<'tc> {
    pub mode: SamplerMode,
    pub texture: &'tc TextureContainer
}

pub enum BindGroupCacheIdentity<'tc> {
    SingleChannel {
        ch_0: BindGroupCacheChannel<'tc>
    },
    DualChannel {
        ch_0: BindGroupCacheChannel<'tc>,
        ch_1: BindGroupCacheChannel<'tc>
    }
}

impl BindGroupCache {
    pub fn create(graphics_provider: &GraphicsProvider) -> Self {
        let samplers = Samplers::create(graphics_provider.get_device());

        let bind_group_layout = graphics_provider.get_device().create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Texture Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: BG0_0_TEXTURE_INDEX,
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
                    binding: BG0_0_SAMPLER_INDEX,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: BG0_1_TEXTURE_INDEX,
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
                    binding: BG0_1_SAMPLER_INDEX,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ]
        });

        return Self {
            bind_group_layout,
            samplers
        };
    }

    pub fn get_texture_layout(&self) -> &BindGroupLayout {
        return &self.bind_group_layout;
    }

    pub fn get(&self,identity: BindGroupCacheIdentity) -> Option<&BindGroup> {
        todo!();
    }
}
