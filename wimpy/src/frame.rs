//#![allow(dead_code,unused_variables)]

use std::{collections::VecDeque, ptr};
use generational_arena::Index;

use crate::{
    area::Area,
    color::Color,
    frame_processor,
    pipeline_management::Pipeline,
    wgpu_interface::WGPUInterface
};

#[derive(Clone,Copy,PartialEq)]
pub enum FrameType {
    Output,
    Normal,
    Texture
}

type FrameIndex = generational_arena::Index;

#[derive(PartialEq)]
pub enum LockStatus {
    FutureUnlock,
    FutureLock,
    Unlocked,
    Locked,
}

pub struct Frame {
    width: u32,
    height: u32,
    index: FrameIndex,
    usage: FrameType,
    command_buffer: VecDeque<FrameCommand>,
    write_lock: LockStatus
}

pub trait FrameInternal {
    fn get_command_buffer(&self) -> &VecDeque<FrameCommand>;
    fn get_size(&self) -> (u32,u32);
    fn get_type(&self) -> FrameType;

    fn get_clear_color(&self) -> Option<wgpu::Color>;
    fn is_writable(&self) -> bool;

    fn create_output(size: (u32,u32),index: Index) -> Self;
    fn create(size: (u32,u32),options: FrameCreationOptions) -> Self;
    fn create_texture(size: (u32,u32),index: Index) -> Self;

    fn get_index(&self) -> Index;
}

#[derive(PartialEq)]
enum SrcDstCmpResult {
    Success,
    CircularReference,
    ReadonlyDestination,
    EmptySource,
    OutputMisuse
}

pub struct FrameCreationOptions {
    pub persistent: bool,
    pub write_once: bool,
    pub index: Index,
}

impl FrameInternal for Frame {

    //TODO: Implement color selection
    fn get_clear_color(&self) -> Option<wgpu::Color> {
        return match self.write_lock {
            LockStatus::FutureUnlock => Some(wgpu::Color::WHITE),
            LockStatus::FutureLock => Some(wgpu::Color::WHITE),
            LockStatus::Unlocked => None,
            LockStatus::Locked => None,
        }
    }

    fn get_index(&self) -> Index {
        return self.index;
    }

    fn get_command_buffer(&self) -> &VecDeque<FrameCommand> {
        return &self.command_buffer;
    }

    fn get_size(&self) -> (u32,u32) {
        return (self.width,self.height);
    }

    fn get_type(&self) -> FrameType {
        return self.usage;
    }

    fn is_writable(&self) -> bool {
        return self.write_lock != LockStatus::Locked;
    }

    fn create(size: (u32,u32),options: FrameCreationOptions) -> Self {
        validate_size(size);
        return Self {
            usage: FrameType::Normal,
            write_lock: match options.write_once {
                true => LockStatus::FutureLock,
                false => match options.persistent {
                    true => LockStatus::FutureUnlock,
                    false => LockStatus::FutureLock,
                },
            },
            index: options.index,
            width: size.0,
            height: size.1,
            command_buffer: VecDeque::default()
        };
    }

    fn create_texture(size: (u32,u32),index: Index) -> Self {
        validate_size(size);
        return Self {
            width: size.0,
            height: size.1,
            index: index,
            usage: FrameType::Texture,
            command_buffer: Default::default(),
            write_lock: LockStatus::Locked,
        }
    }

    fn create_output(size: (u32,u32),index: Index) -> Self {
        validate_size(size);
        return Self {
            usage: FrameType::Output,
            width: size.0,
            height: size.1,
            command_buffer: Default::default(),
            write_lock: LockStatus::FutureLock,
            index,
        };
    }
}

pub enum FrameCommand {

    /* Single Fire Draw Commands */

    DrawColor(PositionColorRotation),

    DrawFrame(FrameIndex,PositionUVRotation),
    DrawFrameColored(FrameIndex,PositionUVColorRotation),

    /* Set Based Draw Commands */

    DrawColorSet(Vec<PositionColorRotation>),

    DrawFrameSet(FrameIndex,Vec<PositionUVRotation>),
    DrawFrameColoredSet(FrameIndex,Vec<PositionUVColorRotation>),

    /* Other */

    SetTextureFilter(FilterMode),
    SetTextureWrap(WrapMode),
}

pub enum WrapMode {
    Clamp,
    Repeat,
    MirrorRepeat
}

pub enum FilterMode {
    Nearest,
    Linear,
}

pub struct PositionColorRotation {
    pub position: Area,
    pub color: Color,
    pub rotation: f32
}

pub struct PositionUVRotation {
    pub position: Area,
    pub uv: Area,
    pub rotation: f32
}

pub struct PositionUVColorRotation {
    pub position: Area,
    pub uv: Area,
    pub color: Color,
    pub rotation: f32
}

fn validate_size(size: (u32,u32)) {
    if size.0 > 0 && size.1 > 0 {
        return;
    }
    panic!("Invalid frame size. Width and height must be greater than 1.");
}

impl Frame {

    /* Internal */

    fn validate_source_destination(&self,source: &Frame) -> SrcDstCmpResult {
        let destination = self;

        if source.usage == FrameType::Output {
            return SrcDstCmpResult::OutputMisuse;
        }

        if match source.write_lock {
            LockStatus::FutureUnlock => true,
            LockStatus::FutureLock => true,
            LockStatus::Unlocked => false,
            LockStatus::Locked => false,
        } {
            return SrcDstCmpResult::EmptySource;
        }

        if !destination.is_writable() {
            return SrcDstCmpResult::ReadonlyDestination;
        }

        if ptr::eq::<Frame>(destination,source) {
            return SrcDstCmpResult::CircularReference;
        }

        return SrcDstCmpResult::Success;
    }

    fn validate(&self,frame: &Frame) -> bool {
        let result = self.validate_source_destination(frame);
        let valid = result == SrcDstCmpResult::Success;
        if !valid {
            log::error!("Frame draw error: {}",match result {
                SrcDstCmpResult::Success => "Success... but not?",
                SrcDstCmpResult::CircularReference => "Source and destination are the same.",
                SrcDstCmpResult::ReadonlyDestination => "Destination is readonly.",
                SrcDstCmpResult::EmptySource => "Frame source is empty/unrendered.",
                SrcDstCmpResult::OutputMisuse => "Cannot use the output frame as a rendering source.",
            });
        }
        return valid;
    }

    /* Draw Related Commands */

    pub fn set_texture_filter(&mut self,filter_mode: FilterMode) {
        self.command_buffer.push_back(FrameCommand::SetTextureFilter(filter_mode));
    }

    pub fn set_texture_wrap(&mut self,wrap_mode: WrapMode) {
        self.command_buffer.push_back(FrameCommand::SetTextureWrap(wrap_mode));
    }

    /* Draw Commands */

    pub fn draw_color(&mut self,parameters: PositionColorRotation) {
        self.command_buffer.push_back(FrameCommand::DrawColor(parameters));
    }

    pub fn draw_color_set(&mut self,parameters: Vec<PositionColorRotation>) {
        self.command_buffer.push_back(FrameCommand::DrawColorSet(parameters));
    }

    pub fn draw_frame(&mut self,frame: &Frame,parameters: PositionUVRotation) {
        if !self.validate(frame) {
            return;
        }
        self.command_buffer.push_back(FrameCommand::DrawFrame(self.index,parameters));
    }

    pub fn draw_frame_set(&mut self,frame: &Frame,parameters: Vec<PositionUVRotation>) {
        if !self.validate(frame) {
            return;
        }
        self.command_buffer.push_back(FrameCommand::DrawFrameSet(self.index,parameters));
    }

    pub fn draw_frame_colored(&mut self,frame: &Frame,parameters: PositionUVColorRotation) {
        if !self.validate(frame) {
            return;
        }
        self.command_buffer.push_back(FrameCommand::DrawFrameColored(self.index,parameters));
    }

    pub fn draw_frame_colored_set(&mut self,frame: &Frame,parameters: Vec<PositionUVColorRotation>) {
        if !self.validate(frame) {
            return;
        }
        self.command_buffer.push_back(FrameCommand::DrawFrameColoredSet(self.index,parameters));
    }

    /* Output & Interop */

    pub fn finish(&mut self,wgpu_interface: &impl WGPUInterface,pipeline: &mut Pipeline) {
        if !self.is_writable() {
            log::error!("Frame is readonly!");
            self.command_buffer.clear();
            return;
        }
        if self.command_buffer.is_empty() {
            log::warn!("Frame command buffer is empty!");
        }        
        frame_processor::render_frame(&self,wgpu_interface,pipeline);

        match self.write_lock {
            LockStatus::FutureUnlock => self.write_lock = LockStatus::Unlocked,
            LockStatus::FutureLock => self.write_lock = LockStatus::Locked,
            _ => {}
        }
        self.command_buffer.clear();
    }

    /* Size Getters */
    
    pub fn width(&self) -> u32 {
        return self.width;
    }

    pub fn height(&self) -> u32 {
        return self.height;
    }

    pub fn size(&self) -> (u32,u32) {
        return (self.width,self.height);
    }
}
