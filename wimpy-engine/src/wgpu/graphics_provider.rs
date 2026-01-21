use wgpu::{
    Device, Instance, Limits, Queue, RequestAdapterError, RequestDeviceError, Surface, SurfaceConfiguration
};

pub struct GraphicsProvider {
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
}

pub struct GraphicsProviderConfig {
    pub instance: Instance,
    pub surface: Surface<'static>,
    pub width: u32,
    pub height: u32,
    pub limits: Limits
}

const MIN_SURFACE_DIMENSION: u32 = 1;
const MAX_SURFACE_DIMENSION: u32 = 8192;

pub fn validate_surface_dimension(value: u32) -> u32 {
    value.max(MIN_SURFACE_DIMENSION).min(MAX_SURFACE_DIMENSION)
}

#[derive(Debug)]
pub enum GraphicsProviderError {
    AdapterCreationError(RequestAdapterError),
    DeviceCreationError(RequestDeviceError),
}

impl GraphicsProvider {
    pub async fn new(config: GraphicsProviderConfig) -> Result<Self,GraphicsProviderError> {

        let adapter = match config.instance.request_adapter(&wgpu::RequestAdapterOptionsBase {
            power_preference: wgpu::PowerPreference::None,
            force_fallback_adapter: false,  
            compatible_surface: Some(&config.surface)
        }).await {
            Ok(value) => value,
            Err(error) => return Err(GraphicsProviderError::AdapterCreationError(error)),
        };

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
            width: validate_surface_dimension(config.width),
            height: validate_surface_dimension(config.height),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 2
        };

        return Ok(Self { surface: config.surface, device, queue, config: surface_config });
    }

    pub fn set_size(&mut self,width: u32,height: u32) {
        self.config.width = validate_surface_dimension(width);
        self.config.height = validate_surface_dimension(height);
        self.surface.configure(&self.device,&self.config);
    }

    pub fn get_device(&self) -> &wgpu::Device {
        return &self.device;
    }

    pub fn get_queue(&self) -> &wgpu::Queue {
        return &self.queue;
    }

    pub fn get_output_format(&self) -> wgpu::TextureFormat {
        return self.config.format;
    }
    
    pub fn get_output_surface(&self) -> Option<wgpu::SurfaceTexture> {
        match self.surface.get_current_texture() {
            Ok(surface) => {
                return Some(surface);
            },
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                log::warn!("WebGPU surface error. Is the surface lost or outdated? Attempting to configure surface again.");
            },
            Err(error) => {
                log::error!("Unable to render: {}",error);
            }
        }
        return None;
    }
}
