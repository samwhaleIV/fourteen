use wgpu::{
    Device, Instance, Limits, Queue, RequestAdapterError, RequestDeviceError, Surface, SurfaceConfiguration, SurfaceError, SurfaceTexture, TextureFormat
};

pub struct GraphicsProvider {
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    max_texture_dimension: u32
}

pub struct GraphicsProviderConfig {
    pub instance: Instance,
    pub surface: Surface<'static>,
    pub limits: Limits
}

#[derive(Debug)]
pub enum GraphicsProviderError {
    AdapterCreationError(RequestAdapterError),
    DeviceCreationError(RequestDeviceError),
}

impl GraphicsProvider {
    pub async fn new(mut config: GraphicsProviderConfig) -> Result<Self,GraphicsProviderError> {
        let adapter = match config.instance.request_adapter(&wgpu::RequestAdapterOptionsBase {
            power_preference: wgpu::PowerPreference::None,
            force_fallback_adapter: false,  
            compatible_surface: Some(&config.surface)
        }).await {
            Ok(value) => value,
            Err(error) => return Err(GraphicsProviderError::AdapterCreationError(error)),
        };

        let max_texture_dimension = adapter.limits().max_texture_dimension_2d;

        config.limits.max_texture_dimension_2d = max_texture_dimension;

        let (device,queue) = match adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            required_limits: config.limits,
            memory_hints: Default::default(),
            trace: wgpu::Trace::Off
        }).await {
            Ok(value) => value,
            Err(error) => return Err(GraphicsProviderError::DeviceCreationError(error)),
        };

        log::info!("LIMITS INFO: min_uniform_buffer_offset_alignment: {}",adapter.limits().min_uniform_buffer_offset_alignment);
        log::info!("LIMITS INFO: max_texture_dimension_2d: {}",adapter.limits().max_texture_dimension_2d);

        let surface_capabilities = config.surface.get_capabilities(&adapter);

        let surface_format = surface_capabilities.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_capabilities.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: 0,
            height: 0,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 2
        };

        return Ok(Self {
            surface: config.surface,
            device,
            queue,
            config:
            surface_config,
            max_texture_dimension
        });
    }

    pub fn set_size(&mut self,width: u32,height: u32) {
        let old_width = self.config.width;
        let old_height = self.config.height;
        
        let new_width = self.get_safe_texture_dimension(width);
        let new_height = self.get_safe_texture_dimension(height);

        if old_width == new_width && old_height == new_height {
            return;
        }

        self.config.width = new_width;
        self.config.height = new_height;

        self.surface.configure(&self.device,&self.config);
    }

    pub fn get_size(&self) -> (u32,u32) {
       return (self.config.width,self.config.height);
    }

    pub fn get_device(&self) -> &Device {
        return &self.device;
    }

    pub fn get_queue(&self) -> &Queue {
        return &self.queue;
    }

    pub fn get_output_format(&self) -> TextureFormat {
        return self.config.format;
    }
    
    pub fn get_output_surface(&self) -> Result<SurfaceTexture,SurfaceError> {
       self.surface.get_current_texture()
    }

    pub fn get_safe_texture_dimension(&self,value: u32) -> u32 {
        return value.max(1).min(self.max_texture_dimension);
    }

    pub fn get_safe_texture_size(&self,value: (u32,u32)) -> (u32,u32) {
        return (
            self.get_safe_texture_dimension(value.0),
            self.get_safe_texture_dimension(value.1)
        );
    }
}
