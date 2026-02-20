use super::*;
use crate::shared::*;

#[derive(Copy,Clone)]
pub struct RestrictedSize {
    pub input: (u32,u32),
    pub output: (u32,u32),
}

#[derive(Copy,Clone)]
pub struct CacheSize {
    pub input: (u32,u32),
    pub output: u32
}

pub struct OutputFrame {
    size: (u32,u32),
    cache_reference: FrameCacheReference,
    clear_color: wgpu::Color
}

#[derive(Clone,Copy,Debug)]
pub struct TextureFrame {
    size: (u32,u32),
    cache_reference: FrameCacheReference,
}

impl TextureFrame {
    pub fn placeholder() -> Self {
        return Self {
            size: (0,0),
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
    fn get_input_size(&self) -> (u32,u32);

    /// The size of the real texture this frame renders to.
    fn get_output_size(&self) -> (u32,u32);

    fn get_uv_scale(&self) -> (f32,f32) {
        let input = self.get_input_size();
        let output = self.get_output_size();

        (
            input.0 as f32 / output.0 as f32,
            input.1 as f32 / output.1 as f32,
        )
    }

    fn width(&self) -> u32 {
        return self.get_input_size().0;
    }

    fn height(&self) -> u32 {
        return self.get_input_size().1;
    }

    fn size(&self) -> (u32,u32) {
        return self.get_input_size();
    }

    fn area(&self) -> WimpyArea {
        let size = self.get_input_size();
        return WimpyArea {
            x: 0.0,
            y: 0.0,
            width: size.0 as f32,
            height: size.1 as f32
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

    fn get_input_size(&self) -> (u32,u32) {
        self.size
    }

    fn get_output_size(&self) -> (u32,u32) {
        self.size
    }
}

impl FrameReference for TextureFrame {
    fn get_ref(&self) -> FrameCacheReference {
        self.cache_reference
    }

    fn get_input_size(&self) -> (u32,u32) {
        self.size
    }

    fn get_output_size(&self) -> (u32,u32) {
        self.size
    }
}

impl FrameReference for TempFrame {
    fn get_ref(&self) -> FrameCacheReference {
        self.cache_reference
    }

    fn get_input_size(&self) -> (u32,u32) {
        self.size.input
    }

    fn get_output_size(&self) -> (u32,u32) {
        (self.size.output,self.size.output)
    }
}

impl FrameReference for LongLifeFrame {
    fn get_ref(&self) -> FrameCacheReference {
        self.cache_reference
    }

    fn get_input_size(&self) -> (u32,u32) {
        self.size.input
    }

    fn get_output_size(&self) -> (u32,u32) {
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
        size: (u32,u32),
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
        size: (u32,u32),
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
