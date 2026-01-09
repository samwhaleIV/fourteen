use std::{
    collections::VecDeque,
    ptr
};

use crate::{
    shared::{
        Area, Color
    },
    wgpu::{
        WGPUHandle,
        graphics_context::GraphicsContextInternal
    }
};

use super::graphics_context::{
    QuadInstance,
    GraphicsContext,
};

use generational_arena::Index;

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
    command_buffer: VecDeque<FrameCommand>, //Look into smallvec...
    write_lock: LockStatus
}

pub trait FrameInternal {
    fn get_command_buffer(&self) -> &VecDeque<FrameCommand>;
    fn get_clear_color(&self) -> Option<wgpu::Color>;
    fn is_writable(&self) -> bool;

    fn create_output(size: (u32,u32),index: Index) -> Self;
    fn create(size: (u32,u32),options: FrameCreationOptions) -> Self;
    fn create_texture(size: (u32,u32),index: Index) -> Self;

    fn get_index(&self) -> Index;

    fn finish<THandle: WGPUHandle>(&mut self,context: &mut GraphicsContext<THandle>);
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
            LockStatus::FutureUnlock => Some(wgpu::Color::BLACK),
            LockStatus::FutureLock => Some(wgpu::Color::BLACK),
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

    fn finish<THandle: WGPUHandle>(&mut self,context: &mut GraphicsContext<THandle>) {
        if !self.is_writable() {
            log::error!("Frame is readonly!");
            self.command_buffer.clear();
            return;
        }
        if self.command_buffer.is_empty() {
            log::warn!("Frame command buffer is empty!");
        }    
        context.render_frame(&self);

        match self.write_lock {
            LockStatus::FutureUnlock => self.write_lock = LockStatus::Unlocked,
            LockStatus::FutureLock => self.write_lock = LockStatus::Locked,
            _ => {}
        }
        self.command_buffer.clear();
    }
}

 //Conveniently enough, 64 bytes wide.
pub enum FrameCommand {
    /* Single Fire Draw Commands */

    DrawFrame(FrameIndex,DrawData),

    /* Set Based Draw Commands */

    DrawFrameSet(FrameIndex,Vec<DrawData>),

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

impl DrawData {
    pub fn to_quad_instance(&self) -> QuadInstance {
        let area = self.area.to_center_encoded();
        return QuadInstance {
            position: [
                area.x,
                area.y,
            ],
            size: [
                area.width,
                area.height,
            ],
            uv_position: [
                self.uv.x,
                self.uv.y,
            ],
            uv_size: [
                self.uv.width,
                self.uv.height,
            ],
            color: self.color.to_float_array(),
            rotation: self.rotation,
            _padding: [0.0,0.0,0.0],
        }
    }
}

fn validate_size(size: (u32,u32)) {
    if size.0 > 0 && size.1 > 0 {
        return;
    }
    panic!("Invalid frame size. Width and height must be greater than 1.");
}

fn get_src_dst_cmp_result(destination: &Frame,source: &Frame) -> SrcDstCmpResult {
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

fn validate_src_dst_op(destination: &Frame,source: &Frame) -> bool {
    let result = get_src_dst_cmp_result(destination,source);
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

impl Frame {

    pub fn set_texture_filter(&mut self,filter_mode: FilterMode) {
        self.command_buffer.push_back(FrameCommand::SetTextureFilter(filter_mode));
    }

    pub fn set_texture_wrap(&mut self,wrap_mode: WrapMode) {
        self.command_buffer.push_back(FrameCommand::SetTextureWrap(wrap_mode));
    }

    /* Draw Commands */
    pub fn draw(&mut self,source_frame: &Frame,draw_data: DrawData) {
        if !validate_src_dst_op(self,source_frame) {
            return;
        }
        self.command_buffer.push_back(FrameCommand::DrawFrame(source_frame.index,draw_data));
    }

    pub fn draw_set(&mut self,source_frame: &Frame,draw_data: Vec<DrawData>) {
        if !validate_src_dst_op(self,source_frame) {
            return;
        }
        self.command_buffer.push_back(FrameCommand::DrawFrameSet(source_frame.index,draw_data));
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
