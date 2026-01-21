use smallvec::SmallVec;

use crate::{
    shared::{
        Area,
        Color
    }, 
    wgpu::frame_cache::FrameCacheReference
};

#[derive(Clone,Copy,PartialEq)]
pub enum FrameType {
    Output,
    Normal,
    Texture
}

#[derive(PartialEq)]
pub enum LockStatus {
    FutureUnlock,
    FutureLock,
    Unlocked,
    Locked,
}

const DEFAULT_COMMAND_BUFFER_SIZE: usize = 32;

pub struct Frame {
    width: u32,
    height: u32,
    cache_reference: FrameCacheReference,
    usage: FrameType,
    command_buffer: SmallVec<[FrameCommand;DEFAULT_COMMAND_BUFFER_SIZE]>,
    write_lock: LockStatus
}

pub trait FrameInternal {
    fn get_clear_color(&self) -> Option<wgpu::Color>;
    fn is_writable(&self) -> bool;

    fn create_output(size: (u32,u32),cache_reference: FrameCacheReference) -> Self;
    fn create(size: (u32,u32),options: FrameCreationOptions) -> Self;
    fn create_texture(size: (u32,u32),cache_reference: FrameCacheReference) -> Self;

    fn get_cache_reference(&self) -> FrameCacheReference;

    fn get_command_buffer(&self) -> Result<&[FrameCommand],FrameError>;
    fn clear(&mut self);
}

#[derive(Debug)]
pub enum FrameError {
    ReadonlyFrame,
    CircularReference,
    ReadonlyDestination,
    EmptySource,
    OutputMisuse
}

pub struct FrameCreationOptions {
    pub persistent: bool,
    pub write_once: bool,
    pub cache_reference: FrameCacheReference,
}

impl FrameInternal for Frame {

    fn get_clear_color(&self) -> Option<wgpu::Color> {
        return match self.write_lock {
            LockStatus::FutureLock | LockStatus::FutureUnlock => Some(wgpu::Color::BLACK),
            LockStatus::Unlocked | LockStatus::Locked => None,
        }
    }

    fn get_cache_reference(&self) -> FrameCacheReference {
        return self.cache_reference;
    }

    fn is_writable(&self) -> bool {
        return self.write_lock != LockStatus::Locked;
    }

    fn create(size: (u32,u32),options: FrameCreationOptions) -> Self {
        return Self {
            usage: FrameType::Normal,
            write_lock: match options.write_once {
                true => LockStatus::FutureLock,
                false => match options.persistent {
                    true => LockStatus::FutureUnlock,
                    false => LockStatus::FutureLock,
                },
            },
            cache_reference: options.cache_reference,
            width: size.0,
            height: size.1,
            command_buffer: Default::default()
        };
    }

    fn create_texture(size: (u32,u32),cache_reference: FrameCacheReference) -> Self {
        return Self {
            width: size.0,
            height: size.1,
            cache_reference,
            usage: FrameType::Texture,
            command_buffer: Default::default(),
            write_lock: LockStatus::Locked,
        }
    }

    fn create_output(size: (u32,u32),cache_reference: FrameCacheReference) -> Self {
        return Self {
            usage: FrameType::Output,
            width: size.0,
            height: size.1,
            command_buffer: Default::default(),
            write_lock: LockStatus::FutureLock,
            cache_reference,
        };
    }

    fn get_command_buffer(&self) -> Result<&[FrameCommand],FrameError> {
        if !self.is_writable() {
            return Err(FrameError::ReadonlyFrame);
        }
        return Ok(&self.command_buffer);
    }

    fn clear(&mut self) {
        match self.write_lock {
            LockStatus::FutureUnlock => self.write_lock = LockStatus::Unlocked,
            LockStatus::FutureLock => self.write_lock = LockStatus::Locked,
            _ => {}
        }
        self.command_buffer.clear();
    }
}

pub enum FrameCommand {
    /* Single Fire Draw Commands */

    DrawFrame(FrameCacheReference,DrawData),

    /* Set Based Draw Commands */

    DrawFrameSet(FrameCacheReference,Vec<DrawData>),

    /* Other */

    SetTextureFilter(FilterMode),
    SetTextureWrap(WrapMode),
}

#[derive(Copy,Clone,PartialEq)]
pub enum WrapMode {
    Clamp,
    Repeat,
    MirrorRepeat
}

#[derive(Copy,Clone,PartialEq)]
pub enum FilterMode {
    Nearest,
    Linear,
}

pub struct DrawData {
    pub area: Area,
    pub uv: Area,
    pub color: Color,
    pub rotation: f32
}

impl Default for DrawData {
    fn default() -> Self {
        Self {
            area: Area::ONE,
            uv: Area::ONE,
            color: Color::WHITE,
            rotation: 0.0
        }
    }
}

fn validate_src_dst_op(destination: &Frame,source: &Frame) -> Result<(),FrameError> {
    if source.usage == FrameType::Output {
        return Err(FrameError::OutputMisuse);
    }

    if match source.write_lock {
        LockStatus::FutureUnlock | LockStatus::FutureLock => true,
        LockStatus::Unlocked | LockStatus::Locked  => false,
    } {
        return Err(FrameError::EmptySource);
    }

    if !destination.is_writable() {
        return Err(FrameError::ReadonlyDestination);
    }

    if destination.cache_reference == source.cache_reference {
        return Err(FrameError::CircularReference);
    }

    return Ok(());
}

impl Frame {

    pub fn set_texture_filter(&mut self,filter_mode: FilterMode) {
        self.command_buffer.push(FrameCommand::SetTextureFilter(filter_mode));
    }

    pub fn set_texture_wrap(&mut self,wrap_mode: WrapMode) {
        self.command_buffer.push(FrameCommand::SetTextureWrap(wrap_mode));
    }

    /* Draw Commands */
    pub fn draw(&mut self,source_frame: &Frame,draw_data: DrawData) -> Result<(),FrameError> {
        validate_src_dst_op(self, source_frame)?;
        self.command_buffer.push(FrameCommand::DrawFrame(source_frame.cache_reference,draw_data));
        return Ok(());
    }

    pub fn draw_set(&mut self,source_frame: &Frame,draw_data: Vec<DrawData>) -> Result<(),FrameError> {
       validate_src_dst_op(self, source_frame)?;
        self.command_buffer.push(FrameCommand::DrawFrameSet(source_frame.cache_reference,draw_data));
        return Ok(());
    }

    /* Size Getters */
    pub fn size(&self) -> (u32,u32) {
        return (self.width,self.height);
    }

    pub fn width(&self) -> u32 {
        return self.width;
    }

    pub fn height(&self) -> u32 {
        return self.height;
    }

    pub fn area(&self) -> Area {
        return Area {
            x: 0.0,
            y: 0.0,
            width: self.width as f32,
            height: self.height as f32,
        }
    }
}
