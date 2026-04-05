use crate::UWimpyPoint;
use super::{WimpyTextureKey, FilteredSize, TextureCacheResolver, SizeInfo};

pub struct Output {
    key:            WimpyTextureKey,
    size:           UWimpyPoint,
    clear_color:    wgpu::Color
}

pub struct Temp {
    key:            WimpyTextureKey,
    size:           FilteredSize,
    clear_color:    wgpu::Color,
}

pub struct LongLife {
    key:        WimpyTextureKey,
    size:       FilteredSize,
}

impl Output {
    pub fn new(
        size:               UWimpyPoint,
        gpu_texture_key:    WimpyTextureKey,
        clear_color:        wgpu::Color,
    ) -> Self {
        Self {
            size,
            key: gpu_texture_key,
            clear_color,
        }
    }
}

impl LongLife {
    pub fn new(
        size:               FilteredSize,
        gpu_texture_key:    WimpyTextureKey,
    ) -> Self {
        Self {
            size,
            key: gpu_texture_key,
        }
    }
}

impl Temp {
    pub fn new(
        size:               FilteredSize,
        gpu_texture_key:    WimpyTextureKey,
        clear_color:        wgpu::Color,
    ) -> Self {
        Self {
            size,
            key: gpu_texture_key,
            clear_color
        }
    }
}

pub trait RenderTarget: SizeInfo + TextureCacheResolver {
    fn get_clear_color(&self) -> Option<wgpu::Color>;
    fn is_output_surface(&self) -> bool;
    fn get_key(&self) -> WimpyTextureKey;
}

impl RenderTarget for Output {
    fn get_clear_color(&self) -> Option<wgpu::Color> {
        Some(self.clear_color)
    }
    fn is_output_surface(&self) -> bool {
        true
    }
    fn get_key(&self) -> WimpyTextureKey {
        self.key
    }
}

impl RenderTarget for Temp {
    fn get_clear_color(&self) -> Option<wgpu::Color> {
        Some(self.clear_color)
    }
    fn is_output_surface(&self) -> bool {
        false
    }
    fn get_key(&self) -> WimpyTextureKey {
        self.key
    }
}

impl RenderTarget for LongLife {
    fn get_clear_color(&self) -> Option<wgpu::Color> {
        None
    }
    fn is_output_surface(&self) -> bool {
        false
    }
    fn get_key(&self) -> WimpyTextureKey {
        self.key
    }
}

impl SizeInfo for Output {
    fn get_input_size(&self) -> UWimpyPoint {
        self.size
    }

    fn get_output_size(&self) -> UWimpyPoint {
        self.size
    }
}

impl SizeInfo for LongLife {
    fn get_input_size(&self) -> UWimpyPoint {
        self.size.input
    }

    fn get_output_size(&self) -> UWimpyPoint {
        self.size.output
    }
}

impl SizeInfo for Temp {
    fn get_input_size(&self) -> UWimpyPoint {
        self.size.input
    }

    fn get_output_size(&self) -> UWimpyPoint {
        self.size.output
    }
}
