use wgpu::{SurfaceTexture, TextureView};

pub trait WGPUInterface {
    fn get_device(&self) -> &wgpu::Device;
    fn get_queue(&self) -> &wgpu::Queue;

    fn get_output_format(&self) -> wgpu::TextureFormat;
    fn get_output(&self) -> Option<OutputResult>;
}

pub struct OutputResult {
    pub surface: SurfaceTexture,
    pub texture_view: TextureView,
    pub size: (u32,u32)
}
