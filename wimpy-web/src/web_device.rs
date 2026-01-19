use wimpy_engine::wgpu::GraphicsProvider;

pub struct WebDevice {

}

impl GraphicsProvider for WebDevice {
    fn get_device(&self) -> &wgpu::Device {
        todo!()
    }

    fn get_queue(&self) -> &wgpu::Queue {
        todo!()
    }

    fn get_output_format(&self) -> wgpu::TextureFormat {
        todo!()
    }

    fn get_output_surface(&self) -> Option<wgpu::SurfaceTexture> {
        todo!()
    }
}
