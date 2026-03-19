pub const UNIFORM_BIND_GROUP_ENTRY_INDEX:   u32 = 0;

pub const UNIFORM_BUFFER_ALIGNMENT:         usize = 256;

pub const DEFAULT_DOUBLE_BUFFER_SIZE:       usize = 1024;

pub const PREFER_SRGB_OUTPUT_SURFACE:       bool = true;
pub const INTERNAL_RENDER_TARGET_FORMAT:    wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

pub const DEPTH_STENCIL_TEXTURE_FORMAT:     wgpu::TextureFormat = wgpu::TextureFormat::Depth24Plus;

pub mod assets {
    pub const FONT_CLASSIC:         &'static str = "wimpy/font/classic";
    pub const FONT_CLASSIC_OUTLINE: &'static str = "wimpy/font/classic-outline";
    pub const FONT_TWELVEN:         &'static str = "wimpy/font/twelven";
    pub const FONT_TWELVEN_SHADED:  &'static str = "wimpy/font/twelven-shaded";
    pub const FONT_MONO_ELF:        &'static str = "wimpy/font/mono-elf";
}
