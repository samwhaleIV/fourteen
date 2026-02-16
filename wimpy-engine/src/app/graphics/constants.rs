pub const UNIFORM_BUFFER_ALIGNMENT: usize = 256;
pub const TEXTURE_BIND_GROUP_INDEX: u32 = 0;
pub const UNIFORM_BIND_GROUP_INDEX: u32 = 1;

pub const BG0_CH0_TEXTURE_INDEX: u32 = 0; //Group 0, index 0
pub const BG0_CH0_SAMPLER_INDEX: u32 = 1; //Group 0, index 1

pub const BG0_CH1_TEXTURE_INDEX: u32 = 2; //Group 0, index 2
pub const BG0_CH1_SAMPLER_INDEX: u32 = 3; //Group 0, index 2

pub const CAMERA_UNIFORM_BIND_GROUP_ENTRY_INDEX: u32 =  0; //Group 1, index 0

pub const DEFAULT_DOUBLE_BUFFER_SIZE: usize = 1024;
pub const DEFAULT_BIND_GROUP_CACHE_SIZE: usize = 64;

// Used for intermediate frames
pub const INTERNAL_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm; //experiment without srgb
