use crate::app::graphics::GraphicsContextConfig;

pub struct TestConfig;

impl GraphicsContextConfig for TestConfig {
    // If a vertex is 32 bytes, there is 31,250 vertices per megabyte.
    const MODEL_CACHE_VERTEX_BUFFER_SIZE: usize = 16384;
    const MODEL_CACHE_INDEX_BUFFER_SIZE: usize = 16384;
    const UNIFORM_BUFFER_SIZE: usize = 16384;
    const INSTANCE_BUFFER_SIZE_2D: usize = 16384;
    const INSTANCE_BUFFER_SIZE_3D: usize = 16384;
    const TEXT_PIPELINE_BUFFER_SIZE: usize = 16384;
    const LINE_BUFFER_SIZE: usize = 16384;
}
