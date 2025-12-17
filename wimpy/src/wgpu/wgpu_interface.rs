pub trait WGPUInterface {
    fn get_device(&self) -> &wgpu::Device;
    fn get_queue(&self) -> &wgpu::Queue;

    fn get_output_format(&self) -> wgpu::TextureFormat;
    fn get_output_surface(&self) -> Option<wgpu::SurfaceTexture>;
}
