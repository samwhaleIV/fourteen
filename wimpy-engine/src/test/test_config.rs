use crate::app::graphics::GraphicsConfig;

pub struct TestConfig;

const BASE: usize = 16384;

impl GraphicsConfig for TestConfig {
    const MESH_CACHE_VERTEX_BUFFER_SIZE: usize = BASE * 4;
    const MESH_CACHE_INDEX_BUFFER_SIZE: usize = BASE * 4;

    const UNIFORM_BUFFER_SIZE: usize = BASE;

    const INSTANCE_BUFFER_SIZE_2D: usize = BASE;
    const INSTANCE_BUFFER_SIZE_3D: usize = BASE;

    const TEXT_PIPELINE_BUFFER_SIZE: usize = BASE;
    const LINE_BUFFER_SIZE: usize = BASE;
}
