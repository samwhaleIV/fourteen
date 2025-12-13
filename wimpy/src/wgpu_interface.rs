use crate::pipeline_management::Pipeline;

pub trait WGPUInterface {
    fn get_device(&self) -> wgpu::Device;
    fn get_queue(&self) -> wgpu::Queue;
    fn get_output_format(&self) -> wgpu::TextureFormat;
    fn get_pipeline(&self) -> &Pipeline;
    fn get_output_size(&self) -> (u32,u32);
    fn get_output_texture(&self) -> wgpu::TextureView;
    fn start_encoding(&mut self);
    fn get_encoder(&self) -> Option<&wgpu::CommandEncoder>;
    fn finish_encoding(&mut self);
}
