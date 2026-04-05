const DEFAULT_BIND_GROUP_CACHE_SIZE: usize = 64;

use wgpu::*;
use std::{collections::HashMap, hash::Hash};

use super::{BindGroupIdentity, SamplerMode};
use crate::app::graphics::constants;

struct FilterSet {
    filter:         FilterMode,
    mipmap_filter:  MipmapFilterMode,
    address:        AddressMode
}

impl FilterSet {
    const fn new(
        filter:         FilterMode,
        mipmap_filter:  MipmapFilterMode,
        address:        AddressMode
    ) -> Self {
        Self {
            filter,
            mipmap_filter,
            address,
        }
    }
    const fn from_sampler_mode(sampler_mode: SamplerMode) -> Self {
        use SamplerMode::*;
        match sampler_mode {
            NearestClamp =>        Self::new(FilterMode::Nearest,   MipmapFilterMode::Nearest,  AddressMode::ClampToEdge),
            NearestWrap =>         Self::new(FilterMode::Nearest,   MipmapFilterMode::Nearest,  AddressMode::Repeat),
            NearestWrapMirror =>   Self::new(FilterMode::Nearest,   MipmapFilterMode::Nearest,  AddressMode::MirrorRepeat),
            LinearClamp =>         Self::new(FilterMode::Linear,    MipmapFilterMode::Linear,   AddressMode::ClampToEdge),
            LinearWrap =>          Self::new(FilterMode::Linear,    MipmapFilterMode::Linear,   AddressMode::Repeat),
            LinearWrapMirror =>    Self::new(FilterMode::Linear,    MipmapFilterMode::Linear,   AddressMode::MirrorRepeat),
        }
    }
}

pub struct Samplers {
    nearest_clamp:          Sampler,
    nearest_wrap:           Sampler,
    nearest_wrap_mirror:    Sampler,
    linear_clamp:           Sampler,
    linear_wrap:            Sampler,
    linear_wrap_mirror:     Sampler
}

impl Samplers {
    fn get(&self,sampler_mode: SamplerMode) -> &Sampler {
        match sampler_mode {
            SamplerMode::NearestClamp =>        &self.nearest_clamp,
            SamplerMode::NearestWrap =>         &self.nearest_wrap,
            SamplerMode::NearestWrapMirror =>   &self.nearest_wrap_mirror,
            SamplerMode::LinearClamp =>         &self.linear_clamp,
            SamplerMode::LinearWrap =>          &self.linear_wrap,
            SamplerMode::LinearWrapMirror =>    &self.linear_wrap_mirror,
        }
    }
}

fn create_sampler(device: &Device,sampler_mode: SamplerMode) -> Sampler {
    let filter_set = FilterSet::from_sampler_mode(sampler_mode);
    device.create_sampler(&SamplerDescriptor {
        address_mode_u:     filter_set.address,
        address_mode_v:     filter_set.address,
        address_mode_w:     filter_set.address,
        mag_filter:         filter_set.filter,
        min_filter:         filter_set.filter,
        mipmap_filter:      filter_set.mipmap_filter, // TODO: figure out if should be fixed to linear
        ..Default::default()
    })
}

impl Samplers {
    pub fn create(device: &Device) -> Self {
        return Self {
            nearest_clamp:          create_sampler(device, SamplerMode::NearestClamp),
            nearest_wrap:           create_sampler(device, SamplerMode::NearestWrap),
            nearest_wrap_mirror:    create_sampler(device, SamplerMode::NearestWrapMirror),
            linear_clamp:           create_sampler(device, SamplerMode::LinearClamp),
            linear_wrap:            create_sampler(device, SamplerMode::LinearWrap),
            linear_wrap_mirror:     create_sampler(device, SamplerMode::LinearWrapMirror),
        };
    }
}

struct Channel<'a> {
    texture: &'a TextureView,
    sampler: &'a Sampler,
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
                binding: constants::CH0_TEXTURE_INDEX,
                resource: BindingResource::TextureView(channel.texture),
            },
            BindGroupEntry {
                binding: constants::CH0_SAMPLER_INDEX,
                resource: BindingResource::Sampler(channel.sampler),
            },
            BindGroupEntry {
                binding: constants::CH1_TEXTURE_INDEX,
                resource: BindingResource::TextureView(channel.texture),
            },
            BindGroupEntry {
                binding: constants::CH1_SAMPLER_INDEX,
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
                binding: constants::CH0_TEXTURE_INDEX,
                resource: BindingResource::TextureView(channel_0.texture),
            },
            BindGroupEntry {
                binding: constants::CH0_SAMPLER_INDEX,
                resource: BindingResource::Sampler(channel_0.sampler),
            },
            BindGroupEntry {
                binding: constants::CH1_TEXTURE_INDEX,
                resource: BindingResource::TextureView(channel_1.texture),
            },
            BindGroupEntry {
                binding: constants::CH1_SAMPLER_INDEX,
                resource: BindingResource::Sampler(channel_1.sampler),
            }
        ],
    });
}

pub struct BindGroupCache {
    samplers:   Samplers,
    layout:     BindGroupLayout,
    cache:      HashMap<CacheKey,BindGroup>,
}

#[derive(Hash,PartialEq,Eq)]
struct CacheKeyChannel {
    mode:   SamplerMode,
    id:     BindGroupIdentity
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

pub struct BindGroupChannel<'a> {
    pub id:             BindGroupIdentity,
    pub sampler_mode:   SamplerMode,
    pub texture_view:   &'a TextureView
}

pub enum BindGroupChannelSet<'a> {
    Single {
        ch_0: BindGroupChannel<'a>
    },
    Dual {
        ch_0: BindGroupChannel<'a>,
        ch_1: BindGroupChannel<'a>
    }
}

impl PartialEq for BindGroupChannel<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.sampler_mode == other.sampler_mode && self.id == other.id
    }
}

impl From<&BindGroupChannelSet<'_>> for CacheKey {
    fn from(value: &BindGroupChannelSet<'_>) -> Self {
        return match value {
            BindGroupChannelSet::Single { ch_0 } =>       Self::SingleChannel { ch_0: ch_0.into() },
            BindGroupChannelSet::Dual   { ch_0, ch_1 } => Self::DualChannel   { ch_0: ch_0.into(), ch_1: ch_1.into() },
        };
    }
}

impl From<&BindGroupChannel<'_>> for CacheKeyChannel {
    fn from(value: &BindGroupChannel<'_>) -> Self {
        return Self {
            mode: value.sampler_mode,
            id: value.id,
        }
    }
}

impl BindGroupCache {
    pub fn create(device: &Device,layout: BindGroupLayout) -> Self {
        let samplers = Samplers::create(device);

        Self {
            samplers,
            layout,
            cache: HashMap::with_capacity(DEFAULT_BIND_GROUP_CACHE_SIZE),
        }
    }

    pub fn get(&mut self,device: &Device,channel_set: &BindGroupChannelSet) -> &BindGroup {
        let entry = self.cache.entry(channel_set.into());
        return entry.or_insert_with(||match channel_set {
            BindGroupChannelSet::Single { ch_0 } =>  create_single_channel_bind_group(
                device,
                &self.layout,
                Channel {
                    texture: &ch_0.texture_view,
                    sampler: self.samplers.get(ch_0.sampler_mode),
                }
            ),
            BindGroupChannelSet::Dual { ch_0, ch_1 } => create_dual_channel_bind_group(
                device,
                &self.layout,
                Channel {
                    texture: &ch_0.texture_view,
                    sampler: self.samplers.get(ch_0.sampler_mode),
                },
                Channel {
                    texture: &ch_1.texture_view,
                    sampler: self.samplers.get(ch_1.sampler_mode),
                }
            ),
        });
    }
}
