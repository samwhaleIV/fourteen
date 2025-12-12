//#![allow(dead_code,unused_variables)]

use std::{collections::VecDeque, ptr};
use crate::{area::Area, color::Color, frame_binder::{FrameBinder, WGPUInterface}};

#[derive(Clone,Copy,PartialEq)]
pub enum FrameUsage {
    Output,
    Mutable,
    Immutable,
    Invalid
}

type FrameIndex = generational_arena::Index;

pub struct Frame {
    width: u32,
    height: u32,
    index: Option<FrameIndex>,
    usage: FrameUsage,
    command_buffer: VecDeque<FrameCommand>,  
}

pub trait FrameInternal {
    fn get_command_buffer(&self) -> &VecDeque<FrameCommand>;
    fn get_size(&self) -> (u32,u32);
    fn get_usage(&self) -> FrameUsage;

    fn to_mutable(size: (u32,u32),index: generational_arena::Index) -> Frame;
    fn to_immutable(size: (u32,u32),index: generational_arena::Index) -> Frame;

    fn is_writable(&self) -> bool;
    fn is_invalid(&self) -> bool;
}

#[derive(PartialEq)]
enum SrcDstCmpResult {
    Success,
    ReadonlyDestination,
    InvalidDestination,
    InvalidSource,
    CircularReference
}

impl FrameInternal for Frame {
    fn get_command_buffer(&self) -> &VecDeque<FrameCommand> {
        return &self.command_buffer;
    }

    fn get_size(&self) -> (u32,u32) {
        return (self.width,self.height);
    }

    fn get_usage(&self) -> FrameUsage {
        return self.usage;
    }

    fn is_writable(&self) -> bool {
        return match self.usage {
            FrameUsage::Output => true,
            FrameUsage::Mutable => self.index.is_none(),
            FrameUsage::Immutable => false,
            FrameUsage::Invalid => false,
        };
    }

    fn is_invalid(&self) -> bool {
        return self.usage == FrameUsage::Invalid || self.index.is_none();
    }

    fn to_mutable(size: (u32,u32),index: generational_arena::Index) -> Frame {
        return Frame {
            width: size.0,
            height: size.1,
            usage: FrameUsage::Mutable,
            index: Some(index),
            command_buffer: Vec::with_capacity(0).into()
        };
    }

    fn to_immutable(size: (u32,u32),index: generational_arena::Index) -> Frame {
        return Frame {
            width: size.0,
            height: size.1,
            usage: FrameUsage::Immutable,
            index: Some(index),
            command_buffer: Vec::with_capacity(0).into()
        };
    }
}

pub enum FrameCommand {

    /* Single Fire Draw Commands */

    DrawColor(PositionColor),

    DrawFrame(FrameIndex,PositionUV),
    DrawFrameColored(FrameIndex,PositionUVColor),

    /* Set Based Draw Commands */

    DrawColorSet(Vec<PositionColor>),

    DrawFrameSet(FrameIndex,Vec<PositionUV>),
    DrawFrameColoredSet(FrameIndex,Vec<PositionUVColor>),

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

pub struct PositionColor {
    position: Area,
    color: Color
}

pub struct PositionUV {
    position: Area,
    uv: Area,
}

pub struct PositionUVColor {
    position: Area,
    uv: Area,
    color: Color
}

fn validate_size(width: u32,height: u32) {
    if width > 0 && height > 0 {
        return;
    }
    panic!("Invalid frame size. Width and height must be greater than 1.");
}

impl Frame {

    /* Internal */

    fn validate_source_destination(&self,source: &Frame) -> SrcDstCmpResult {
        let destination = self;

        if source.is_invalid() {
            return SrcDstCmpResult::InvalidSource;
        }

        /* These are redunant with Frame.finish(). */
        if destination.is_invalid() {
            return SrcDstCmpResult::InvalidDestination;
        }
        if destination.is_writable() {
            return SrcDstCmpResult::ReadonlyDestination;
        }
        /* - - - - - - - - - - - - - - - - - - - - */

        if ptr::eq::<Frame>(destination,source) {
            return SrcDstCmpResult::CircularReference;
        }
        return SrcDstCmpResult::Success;
    }

    fn validate(&self,frame: &Frame) -> bool {
        let result = self.validate_source_destination(frame);
        let valid = result == SrcDstCmpResult::Success;
        if !valid {
            log::error!("Frame draw error: {}.",match result {
                SrcDstCmpResult::ReadonlyDestination => "Destination frame is readonly",
                SrcDstCmpResult::InvalidDestination => "Destination frame is null/invalid",
                SrcDstCmpResult::InvalidSource => "Source frame is null/invalid",
                SrcDstCmpResult::CircularReference => "Source frame is the same as the destination frame",
                _ => "Unknown"
            });
        }
        return valid;
    }

    /* Creation */

    fn create_null() -> Self {
        return Self {
            usage: FrameUsage::Invalid,
            index: None,
            width: 0,
            height: 0,
            command_buffer: Vec::with_capacity(0).into()
        }
    }

    pub fn create(width: u32,height: u32) -> Self {
        validate_size(width,height);
        return Self {
            usage: FrameUsage::Immutable,
            index: None,
            width,
            height,
            command_buffer: VecDeque::default()
        };
    }
    
    pub fn create_mutable(width: u32,height: u32) -> Self {
        validate_size(width,height);
        return Self {
            usage: FrameUsage::Mutable,
            index: None,
            width,
            height,
            command_buffer: VecDeque::default()
        };
    }

    pub fn create_output(wgpu_interface: &impl WGPUInterface) -> Self {
        let (width,height) = wgpu_interface.get_output_size();
        validate_size(width,height);
        return Self {
            usage: FrameUsage::Output,
            index: None,
            width,
            height,
            command_buffer: VecDeque::default()
        };
    }

    /* Draw Related Commands */

    pub fn set_texture_filter(&mut self,filter_mode: FilterMode) {
        self.command_buffer.push_back(FrameCommand::SetTextureFilter(filter_mode));
    }

    pub fn set_texture_wrap(&mut self,wrap_mode: WrapMode) {
        self.command_buffer.push_back(FrameCommand::SetTextureWrap(wrap_mode));
    }

    /* Draw Commands */

    pub fn draw_color(&mut self,parameters: PositionColor) {
        self.command_buffer.push_back(FrameCommand::DrawColor(parameters));
    }

    pub fn draw_color_set(&mut self,parameters: Vec<PositionColor>) {
        self.command_buffer.push_back(FrameCommand::DrawColorSet(parameters));
    }

    pub fn draw_frame(&mut self,frame: &Frame,parameters: PositionUV) {
        if !self.validate(frame) {
            return;
        }
        if let Some(index) = self.index {
            self.command_buffer.push_back(FrameCommand::DrawFrame(index,parameters));
        }
    }

    pub fn draw_frame_set(&mut self,frame: &Frame,parameters: Vec<PositionUV>) {
        if !self.validate(frame) {
            return;
        }
        if let Some(index) = self.index {            
            self.command_buffer.push_back(FrameCommand::DrawFrameSet(index,parameters));
        }
    }

    pub fn draw_frame_colored(&mut self,frame: &Frame,parameters: PositionUVColor) {
        if !self.validate(frame) {
            return;
        }
        if let Some(index) = self.index {
            self.command_buffer.push_back(FrameCommand::DrawFrameColored(index,parameters));
        }
    }

    pub fn draw_frame_colored_set(&mut self,frame: &Frame,parameters: Vec<PositionUVColor>) {
        if !self.validate(frame) {
            return;
        }
        if let Some(index) = self.index {
            self.command_buffer.push_back(FrameCommand::DrawFrameColoredSet(index,parameters));
        }
    }

    /* Output & Interop */

    pub fn finish(&mut self,frame_binder: &mut FrameBinder,wgpu_interface: &impl WGPUInterface) -> Frame {
        let invalid = self.is_invalid();
        if invalid || !self.is_writable() {
            log::error!("Frame render error: Can't render to a {} destination frame.",match invalid {
                true => "null/invalid",
                false => "readonly"
            });
            self.command_buffer.clear();
            return Frame::create_null();
        }
        if self.command_buffer.is_empty() {
            log::warn!("Frame command buffer is empty!");
        }
        let size = self.size();
        let frame = frame_binder.render_frame(&self,wgpu_interface);
        self.command_buffer.clear();
        if self.usage == FrameUsage::Output {
            self.usage = FrameUsage::Invalid;
        }
        return frame;
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
