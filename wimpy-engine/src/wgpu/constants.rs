pub struct BindGroupIndices;

impl BindGroupIndices {
    pub const TEXTURE: u32 = 0;
    pub const UNIFORM: u32 = 1;
}

pub const UNIFORM_BUFFER_ALIGNMENT: usize = 256;
pub const INDEX_BUFFER_SIZE: u32 = 6;

pub const DEFAULT_COMMAND_BUFFER_SIZE: usize = 32;
