pub trait WGPUInterface {
    fn get_device(&self) -> &wgpu::Device;
    fn get_queue(&self) -> &wgpu::Queue;
    fn get_output_texture(&self) -> &wgpu::TextureView;
    fn get_output_format(&self) -> wgpu::TextureFormat;
    fn get_output_size(&self) -> (u32,u32);
}
