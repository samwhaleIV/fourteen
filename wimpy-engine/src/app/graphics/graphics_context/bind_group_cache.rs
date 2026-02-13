use super::prelude::*;

use std::{
    collections::HashMap,
    hash::Hash
};

#[derive(PartialEq,Eq,Copy,Clone,Hash)]
pub enum SamplerMode {
    NearestClamp,
    NearestWrap,
    NearestWrapMirror,
    LinearClamp,
    LinearWrap,
    LinearWrapMirror
}

const fn get_filter_and_address(sampler_mode: SamplerMode) -> (FilterMode,AddressMode) {
    return match sampler_mode {
        SamplerMode::NearestClamp =>        (FilterMode::Nearest,AddressMode::ClampToEdge),
        SamplerMode::NearestWrap =>         (FilterMode::Nearest,AddressMode::Repeat),
        SamplerMode::NearestWrapMirror =>   (FilterMode::Nearest,AddressMode::MirrorRepeat),
        SamplerMode::LinearClamp =>         (FilterMode::Linear,AddressMode::ClampToEdge),
        SamplerMode::LinearWrap =>          (FilterMode::Linear,AddressMode::Repeat),
        SamplerMode::LinearWrapMirror =>    (FilterMode::Linear,AddressMode::MirrorRepeat),
    }
}

pub struct Samplers {
    nearest_clamp: Sampler,
    nearest_wrap: Sampler,
    nearest_wrap_mirror: Sampler,
    linear_clamp: Sampler,
    linear_wrap: Sampler,
    linear_wrap_mirror: Sampler
}

impl Samplers {
    fn get(&self,sampler_mode: SamplerMode) -> &Sampler {
        return match sampler_mode {
            SamplerMode::NearestClamp => &self.nearest_clamp,
            SamplerMode::NearestWrap => &self.nearest_wrap,
            SamplerMode::NearestWrapMirror => &self.nearest_wrap_mirror,
            SamplerMode::LinearClamp => &self.linear_clamp,
            SamplerMode::LinearWrap => &self.linear_wrap,
            SamplerMode::LinearWrapMirror => &self.linear_wrap_mirror,
        }
    }
}

fn create_sampler(device: &Device,sampler_mode: SamplerMode) -> Sampler {
    let (filter,address) = get_filter_and_address(sampler_mode);
    device.create_sampler(&SamplerDescriptor {
        address_mode_u: address,
        address_mode_v: address,
        address_mode_w: address,
        mag_filter: filter,
        min_filter: filter,
        mipmap_filter: filter,
        ..Default::default()
    })
}

impl Samplers {
    pub fn create(device: &Device) -> Self {
        return Self {
            nearest_clamp: create_sampler(device,
                SamplerMode::NearestClamp
            ),
            nearest_wrap: create_sampler(device,
                SamplerMode::NearestWrap
            ),
            nearest_wrap_mirror: create_sampler(device,
                SamplerMode::NearestWrapMirror
            ),
            linear_clamp: create_sampler(device,
                SamplerMode::LinearClamp
            ),
            linear_wrap: create_sampler(device,
                SamplerMode::LinearWrap
            ),
            linear_wrap_mirror: create_sampler(device,
                SamplerMode::LinearWrapMirror
            ),
        };
    }
}
struct Channel<'a> {
    texture: &'a TextureView,
    sampler: &'a Sampler
}

fn create_single_channel_bind_group(
    device: &Device,
    layout: &BindGroupLayout,
    channel: Channel
) -> BindGroup {
    return device.create_bind_group(&BindGroupDescriptor {
        label: Some("Texture Bind Group"),
        layout,
        entries: &[
            BindGroupEntry {
                binding: BG0_CH0_TEXTURE_INDEX,
                resource: BindingResource::TextureView(channel.texture),
            },
            BindGroupEntry {
                binding: BG0_CH0_SAMPLER_INDEX,
                resource: BindingResource::Sampler(channel.sampler),
            }
        ],
    });
}

fn create_dual_channel_bind_group(
    device: &Device,
    layout: &BindGroupLayout,
    channel_0: Channel,
    channel_1: Channel,
) -> BindGroup {
    return device.create_bind_group(&BindGroupDescriptor {
        label: Some("Texture Bind Group"),
        layout,
        entries: &[
            BindGroupEntry {
                binding: BG0_CH0_TEXTURE_INDEX,
                resource: BindingResource::TextureView(channel_0.texture),
            },
            BindGroupEntry {
                binding: BG0_CH0_SAMPLER_INDEX,
                resource: BindingResource::Sampler(channel_0.sampler),
            },
            BindGroupEntry {
                binding: BG0_CH1_TEXTURE_INDEX,
                resource: BindingResource::TextureView(channel_1.texture),
            },
            BindGroupEntry {
                binding: BG0_CH1_SAMPLER_INDEX,
                resource: BindingResource::Sampler(channel_1.sampler),
            }
        ],
    });
}

pub struct BindGroupCache {
    samplers: Samplers,
    cache: HashMap<CacheKey,BindGroup>,
    layout: BindGroupLayout,
}

#[derive(Hash,PartialEq,Eq)]
struct CacheKeyChannel {
    mode: SamplerMode,
    id: TextureContainerIdentity
}

#[derive(Hash,PartialEq,Eq)]
enum CacheKey {
    SingleChannel {
        ch_0: CacheKeyChannel,
    },
    DualChannel {
        ch_0: CacheKeyChannel,
        ch_1: CacheKeyChannel,
    }
}

pub struct BindGroupChannelConfig<'tc> {
    pub mode: SamplerMode,
    pub texture: &'tc TextureContainer
}

pub enum BindGroupCacheIdentity<'tc> {
    SingleChannel {
        ch_0: BindGroupChannelConfig<'tc>
    },
    DualChannel {
        ch_0: BindGroupChannelConfig<'tc>,
        ch_1: BindGroupChannelConfig<'tc>
    }
}

impl PartialEq for BindGroupChannelConfig<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.mode == other.mode && self.texture.get_identity() == other.texture.get_identity()
    }
}

impl From<&BindGroupCacheIdentity<'_>> for CacheKey {
    fn from(value: &BindGroupCacheIdentity<'_>) -> Self {
        return match value {
            BindGroupCacheIdentity::SingleChannel { ch_0 } => Self::SingleChannel { ch_0: ch_0.into() },
            BindGroupCacheIdentity::DualChannel { ch_0, ch_1 } => Self::DualChannel { ch_0: ch_0.into(), ch_1: ch_1.into() },
        };
    }
}

impl From<&BindGroupChannelConfig<'_>> for CacheKeyChannel {
    fn from(value: &BindGroupChannelConfig<'_>) -> Self {
        return Self {
            mode: value.mode,
            id: value.texture.get_identity(),
        }
    }
}

impl BindGroupCache {
    pub fn create(graphics_provider: &GraphicsProvider) -> Self {
        let samplers = Samplers::create(graphics_provider.get_device());

        let bind_group_layout = graphics_provider.get_device().create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Texture Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: BG0_CH0_TEXTURE_INDEX,
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
                    binding: BG0_CH0_SAMPLER_INDEX,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: BG0_CH1_TEXTURE_INDEX,
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
                    binding: BG0_CH1_SAMPLER_INDEX,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ]
        });

        return Self {
            layout: bind_group_layout,
            samplers,
            cache: HashMap::with_capacity(DEFAULT_BIND_GROUP_CACHE_SIZE),
        };
    }

    pub fn get_texture_layout(&self) -> &BindGroupLayout {
        return &self.layout;
    }

    pub fn get(&mut self,device: &Device,identity: &BindGroupCacheIdentity) -> &BindGroup {
        let entry = self.cache.entry(identity.into());
        return entry.or_insert_with(||match identity {
            BindGroupCacheIdentity::SingleChannel { ch_0 } =>  create_single_channel_bind_group(
                device,
                &self.layout,
                Channel {
                    texture: ch_0.texture.get_view(),
                    sampler: self.samplers.get(ch_0.mode),
                }
            ),
            BindGroupCacheIdentity::DualChannel { ch_0, ch_1 } => create_dual_channel_bind_group(
                device,
                &self.layout,
                Channel {
                    texture: ch_0.texture.get_view(),
                    sampler: self.samplers.get(ch_0.mode),
                },
                Channel {
                    texture: ch_1.texture.get_view(),
                    sampler: self.samplers.get(ch_1.mode),
                }
            ),
        });
    }
}
