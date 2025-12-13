pub trait WGPUInterface {
    fn get_device(&self) -> &wgpu::Device;
    fn get_queue(&self) -> &wgpu::Queue;

    fn get_output(&self) -> (wgpu::TextureView,(u32,u32));
    fn get_output_format(&self) -> wgpu::TextureFormat;
}
