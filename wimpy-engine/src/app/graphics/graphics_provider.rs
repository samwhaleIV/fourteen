use wgpu::*;
use crate::UWimpyPoint;
use super::{SizeValidationError, constants};

pub struct GraphicsProvider {
    surface: Surface<'static>,
    device: Device, // TODO: Restrict access
    queue: Queue, // TODO: Restrict access
    config: SurfaceConfiguration,
    max_texture_dimension: u32,
    output_view_format: TextureFormat,
    max_texture_power_of_two: u32,
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
        let adapter = match config.instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&config.surface)
        }).await {
            Ok(value) => value,
            Err(error) => return Err(GraphicsProviderError::AdapterCreationError(error)),
        };

        let max_texture_dimension = adapter.limits().max_texture_dimension_2d;
        let max_uniform_buffer_size = adapter.limits().max_uniform_buffer_binding_size;

        config.limits.max_texture_dimension_2d = max_texture_dimension;
        config.limits.max_uniform_buffer_binding_size = max_uniform_buffer_size;

        let (device,queue) = match adapter.request_device(&DeviceDescriptor {
            label: None,
            required_features: Features::empty(),
            experimental_features: ExperimentalFeatures::disabled(),
            required_limits: config.limits,
            memory_hints: Default::default(),
            trace: Trace::Off
        }).await {
            Ok(value) => value,
            Err(error) => return Err(GraphicsProviderError::DeviceCreationError(error)),
        };

        log::info!("LIMITS INFO: min_uniform_buffer_offset_alignment: {}",adapter.limits().min_uniform_buffer_offset_alignment);
        log::info!("LIMITS INFO: max_texture_dimension_2d: {}",adapter.limits().max_texture_dimension_2d);
        log::info!("LIMITS INFO: max_uniform_buffer_size: {}",adapter.limits().max_uniform_buffer_binding_size);

        let surface_capabilities = config.surface.get_capabilities(&adapter);
        log::info!("Available surface formats: {:?}",surface_capabilities.formats);

        use constants::PREFER_SRGB_OUTPUT_SURFACE as prefer_srgb;

        let primary_format = surface_capabilities.formats.iter()
            .find(|f| f.is_srgb() == prefer_srgb)
            .copied()
            .unwrap_or(surface_capabilities.formats[0]);

        log::info!("Selected surface format: {:?}",primary_format);

        let mut view_formats = Vec::with_capacity(1);

        let desired_format: TextureFormat = {
            if prefer_srgb {
                if primary_format.is_srgb() {
                    primary_format
                } else {
                    let desired_format = primary_format.add_srgb_suffix();
                    view_formats.push(desired_format);
                    desired_format
                }
            } else {
                if !primary_format.is_srgb() {
                    primary_format
                } else {
                    let desired_format = primary_format.remove_srgb_suffix();
                    view_formats.push(desired_format);
                    desired_format
                }
            }
        };

        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: primary_format,
            width: 0,
            height: 0,
            present_mode: PresentMode::AutoVsync,
            alpha_mode: CompositeAlphaMode::Opaque,
            view_formats,
            desired_maximum_frame_latency: 2
        };

        let max_texture_power_of_two = prev_power_of_two(max_texture_dimension);

        Ok(Self {
            surface: config.surface,
            device,
            queue,
            config: surface_config,
            max_texture_dimension,
            max_texture_power_of_two,
            output_view_format: desired_format,
        })
    }

    pub fn set_size(&mut self,width: u32,height: u32) {
        let old_width = self.config.width;
        let old_height = self.config.height;
     
        let new_width = self.get_safe_texture_dimension_value(width);
        let new_height = self.get_safe_texture_dimension_value(height);

        if old_width == new_width && old_height == new_height {
            return;
        }

        self.config.width = new_width;
        self.config.height = new_height;

        self.surface.configure(&self.device,&self.config);
    }

    pub fn get_size(&self) -> UWimpyPoint {
       [self.config.width,self.config.height].into()
    }

    //TODO: Deprecate
    pub fn get_device(&self) -> &Device {
        &self.device
    }

    //TODO: Deprecate
    pub fn get_queue(&self) -> &Queue {
        &self.queue
    }

    /// Not the format of the surface itself, but rather, a view of it.
    /// 
    /// On most platforms, these will be the same. However, in WebGPU, concessions may take place.
    pub fn get_output_view_format(&self) -> TextureFormat {
        self.output_view_format
    }

    pub fn get_output_surface(&self) -> Result<SurfaceTexture,SurfaceError> {
       self.surface.get_current_texture()
    }

    pub fn get_safe_texture_dimension_value(&self,value: u32) -> u32 {
        value.clamp(1,self.max_texture_dimension)
    }

    pub fn get_safe_texture_power_of_two(&self,value: u32) -> u32 {
        value.clamp(1,self.max_texture_power_of_two)
    }

    pub fn get_safe_texture_size(&self,value: UWimpyPoint) -> UWimpyPoint {
        [
            self.get_safe_texture_dimension_value(value.x),
            self.get_safe_texture_dimension_value(value.y)
        ].into()
    }

    pub fn max_texture_dimension_value(&self) -> u32 {
        self.max_texture_dimension
    }

    pub fn max_texture_power_of_two(&self) -> u32 {
        self.max_texture_power_of_two
    }

    pub fn validate_size(&self,size: UWimpyPoint) -> Result<(),SizeValidationError> {
        use SizeValidationError::*;
        let upper_bound = self.max_texture_dimension;
        if size.x < 1 || size.y < 1 {
            Err(TooSmall {
                value: 0,
                limit: 1
            })
        } else if size.x > upper_bound {
            Err(TooBig {
                value: size.x,
                limit: upper_bound
            })
        } else if size.y > upper_bound {
            Err(TooBig {
                value: size.y,
                limit: upper_bound
            })
        } else {
            Ok(())
        }
    }
}

const fn prev_power_of_two(value: u32) -> u32 {
    if value.is_power_of_two() {
        value
    } else {
        value.next_power_of_two() >> 1
    }
}
