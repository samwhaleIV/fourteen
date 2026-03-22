pub const UNIFORM_BIND_GROUP_ENTRY_INDEX:   u32 = 0;

pub const UNIFORM_BUFFER_ALIGNMENT:         usize = 256;

pub const DEFAULT_DOUBLE_BUFFER_SIZE:       usize = 1024;

pub const PREFER_SRGB_OUTPUT_SURFACE:       bool = true;

pub const INTERNAL_RENDER_TARGET_FORMAT:    wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
pub const DEPTH_STENCIL_TEXTURE_FORMAT:     wgpu::TextureFormat = wgpu::TextureFormat::Depth24Plus;

pub const CH0_TEXTURE_INDEX: u32 = 0;
pub const CH0_SAMPLER_INDEX: u32 = 1;

pub const CH1_TEXTURE_INDEX: u32 = 2;
pub const CH1_SAMPLER_INDEX: u32 = 3;
