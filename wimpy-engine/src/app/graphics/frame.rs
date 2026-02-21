use super::*;
use crate::*;

#[derive(Copy,Clone)]
pub struct RestrictedSize {
    pub input: UWimpyPoint,
    pub output: UWimpyPoint,
}

#[derive(Copy,Clone)]
pub struct CacheSize {
    pub input: UWimpyPoint,
    pub output_single_dimension: u32
}

pub struct OutputFrame {
    size: UWimpyPoint,
    cache_reference: FrameCacheReference,
    clear_color: wgpu::Color
}

#[derive(Clone,Copy,Debug)]
pub struct TextureFrame {
    size: UWimpyPoint,
    cache_reference: FrameCacheReference,
}

impl TextureFrame {
    pub fn placeholder() -> Self {
        return Self {
            size: UWimpyPoint::ZERO,
            cache_reference: Default::default(),
        }
    }
}

pub struct TempFrame {
    size: CacheSize,
    cache_reference: FrameCacheReference,
    clear_color: wgpu::Color,
}

pub struct LongLifeFrame {
    size: RestrictedSize,
    cache_reference: FrameCacheReference,
}

pub trait FrameReference {
    fn get_ref(&self) -> FrameCacheReference;

    /// The size of the frame as requested by the user. In the case of an imported texture frame, this is its original size.
    fn get_input_size(&self) -> UWimpyPoint;

    /// The size of the real texture this frame renders to.
    fn get_output_size(&self) -> UWimpyPoint;

    fn get_uv_scale(&self) -> WimpyVec {
        let input = self.get_input_size();
        let output = self.get_output_size();

        WimpyVec::from(input) / WimpyVec::from(output)
    }

    fn width(&self) -> u32 {
        return self.get_input_size().x;
    }

    fn height(&self) -> u32 {
        return self.get_input_size().y;
    }

    fn size(&self) -> UWimpyPoint {
        return self.get_input_size();
    }

    fn area(&self) -> WimpyRect {
        WimpyRect {
            position: WimpyVec::ZERO,
            size: self.get_input_size().into()
        }
    }
}

pub trait MutableFrame: FrameReference {
    fn get_clear_color(&self) -> Option<wgpu::Color>;
    fn is_output_surface(&self) -> bool;
}

impl FrameReference for OutputFrame {
    fn get_ref(&self) -> FrameCacheReference {
        self.cache_reference
    }

    fn get_input_size(&self) -> UWimpyPoint {
        self.size
    }

    fn get_output_size(&self) -> UWimpyPoint {
        self.size
    }
}

impl FrameReference for TextureFrame {
    fn get_ref(&self) -> FrameCacheReference {
        self.cache_reference
    }

    fn get_input_size(&self) -> UWimpyPoint {
        self.size
    }

    fn get_output_size(&self) -> UWimpyPoint {
        self.size
    }
}

impl FrameReference for TempFrame {
    fn get_ref(&self) -> FrameCacheReference {
        self.cache_reference
    }

    fn get_input_size(&self) -> UWimpyPoint {
        self.size.input
    }

    fn get_output_size(&self) -> UWimpyPoint {
        self.size.output_single_dimension.into()
    }
}

impl FrameReference for LongLifeFrame {
    fn get_ref(&self) -> FrameCacheReference {
        self.cache_reference
    }

    fn get_input_size(&self) -> UWimpyPoint {
        self.size.input
    }

    fn get_output_size(&self) -> UWimpyPoint {
        self.size.output
    }
}

impl MutableFrame for OutputFrame {
    fn get_clear_color(&self) -> Option<wgpu::Color> {
        Some(self.clear_color)
    }
    fn is_output_surface(&self) -> bool {
        return true;
    }
}

impl MutableFrame for TempFrame {
    fn get_clear_color(&self) -> Option<wgpu::Color> {
        Some(self.clear_color)
    }
    fn is_output_surface(&self) -> bool {
        return false;
    }
}

impl MutableFrame for LongLifeFrame {
    fn get_clear_color(&self) -> Option<wgpu::Color> {
        None
    }
    fn is_output_surface(&self) -> bool {
        return false;
    }
}

pub struct FrameFactory;

impl FrameFactory {

    pub fn create_output(
        size: UWimpyPoint,
        cache_reference: FrameCacheReference,
        clear_color: wgpu::Color,
    ) -> OutputFrame {
        OutputFrame {
            size,
            cache_reference,
            clear_color,
        }
    }

    pub fn create_texture(
        size: UWimpyPoint,
        cache_reference: FrameCacheReference,
    ) -> TextureFrame {
        TextureFrame {
            size,
            cache_reference,
        }
    }

    pub fn create_long_life(
        size: RestrictedSize,
        cache_reference: FrameCacheReference,
    ) -> LongLifeFrame {
        LongLifeFrame {
            size,
            cache_reference,
        }
    }

    pub fn create_temp_frame(
        size: CacheSize,
        cache_reference: FrameCacheReference,
        clear_color: wgpu::Color,
    ) -> TempFrame {
        TempFrame {
            size,
            cache_reference,
            clear_color
        }
    }
}
