use std::sync::Arc;
use winit::window::Window;
use wimpy::wgpu_interface::WGPUInterface;

pub struct Graphics {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
}

impl Graphics {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Graphics> {

        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window)?;

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptionsBase {
            power_preference: wgpu::PowerPreference::None,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface)
        }).await?;

        let (device,queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            required_limits: wgpu::Limits::default(),
            memory_hints: Default::default(),
            trace: wgpu::Trace::Off
        }).await?;

        let surface_capabilities = surface.get_capabilities(&adapter);

        let surface_format = surface_capabilities.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 2
        };

        return Ok(Self { surface, device, queue, config });
    }

    pub fn configure_surface_size(&mut self,width: u32,height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device,&self.config);
    }
}

impl WGPUInterface for Graphics {
    fn get_device(&self) -> &wgpu::Device {
        return &self.device;
    }

    fn get_queue(&self) -> &wgpu::Queue {
        return &self.queue;
    }

    fn get_output_format(&self) -> wgpu::TextureFormat {
        return self.config.format;
    }
    
    fn get_output_surface(&self) -> Option<wgpu::SurfaceTexture> {
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
