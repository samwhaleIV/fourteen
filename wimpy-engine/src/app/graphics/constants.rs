pub const UNIFORM_BUFFER_ALIGNMENT: usize = 256;

pub const CH0_TEXTURE_INDEX: u32 = 0;
pub const CH0_SAMPLER_INDEX: u32 = 1;

pub const CH1_TEXTURE_INDEX: u32 = 2;
pub const CH1_SAMPLER_INDEX: u32 = 3;

pub const UNIFORM_BIND_GROUP_ENTRY_INDEX: u32 =  0;

pub const DEFAULT_DOUBLE_BUFFER_SIZE: usize = 1024;
pub const DEFAULT_BIND_GROUP_CACHE_SIZE: usize = 64;

pub const PREFER_SRGB_OUTPUT_SURFACE: bool = true;
pub const INTERNAL_RENDER_TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb; // We can bump this up in the future

pub mod assets {
    pub const FONT_CLASSIC:             &'static str = "wimpy/font/classic";
    pub const FONT_CLASSIC_OUTLINE:     &'static str = "wimpy/font/classic-outline";
    pub const FONT_TWELVEN:             &'static str = "wimpy/font/twelven";
    pub const FONT_TWELVEN_SHADED:      &'static str = "wimpy/font/twelven-shaded";
    pub const FONT_MONO_ELF:            &'static str = "wimpy/font/mono-elf";
}
