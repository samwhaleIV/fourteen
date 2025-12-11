#![allow(dead_code,unused_variables)]

use std::collections::VecDeque;
use crate::{area::Area, color::Color, frame_binder::{FrameBinder, WGPUInterface}};

#[derive(Clone,Copy)]
pub enum FrameUsage {
    Output,
    RenderOnce,
    Reuse
}

pub struct Frame {
    width: u32,
    height: u32,
    usage: FrameUsage,
    command_buffer: VecDeque<FrameCommand>,  
}

pub trait FrameInternal {
    fn get_command_buffer(&self) -> &VecDeque<FrameCommand>;
    fn get_size(&self) -> (u32,u32);
    fn get_usage(&self) -> FrameUsage;
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
}

pub struct FinishedFrame {
    width: u32,
    height: u32,
    readonly: bool,
    index: generational_arena::Index,
}

pub trait FinishedFrameInternal {
    fn create_mutable(size: (u32,u32),index: generational_arena::Index) -> FinishedFrame;
    fn create_immutable(size: (u32,u32),index: generational_arena::Index) -> FinishedFrame;
}

impl FinishedFrameInternal for FinishedFrame {
    fn create_mutable(size: (u32,u32),index: generational_arena::Index) -> FinishedFrame {
        return FinishedFrame {
            width: size.0,
            height: size.1,
            readonly: false,
            index,
        };
    }
    fn create_immutable(size: (u32,u32),index: generational_arena::Index) -> FinishedFrame {
        return FinishedFrame {
            width: size.0,
            height: size.1,
            readonly: true,
            index,
        };
    }
}

pub enum FrameCommand {

    /* Single Fire Draw Commands */

    DrawColor(PositionColor),

    DrawFrame(FinishedFrame,PositionUV),
    DrawFrameColored(FinishedFrame,PositionUVColor),

    /* Set Based Draw Commands */

    DrawColorSet(Vec<PositionColor>),

    DrawFrameSet(FinishedFrame,Vec<PositionUV>),
    DrawFrameColoredSet(FinishedFrame,Vec<PositionUVColor>),

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

impl Frame {
    /* Creation */

    pub fn create(width: u32,height: u32) -> Frame {
        return Frame {
            usage: FrameUsage::RenderOnce,
            width,
            height,
            command_buffer: VecDeque::default()
        };
    }

    pub fn create_reusable(wgpu_interface: &impl WGPUInterface) -> Frame {
        let (width,height) = wgpu_interface.get_output_size();
        return Frame {
            usage: FrameUsage::Reuse,
            width,
            height,
            command_buffer: VecDeque::default()
        };
    }

    pub fn create_output(wgpu_interface: &impl WGPUInterface) -> Frame {
        let (width,height) = wgpu_interface.get_output_size();
        return Frame {
            usage: FrameUsage::Output,
            width,
            height,
            command_buffer: VecDeque::default()
        };
    }

    /* Other Commands */
    pub fn set_texture_filter(&mut self,filter_mode: FilterMode) {
        self.command_buffer.push_back(FrameCommand::SetTextureFilter(filter_mode));
    }

    pub fn set_texture_wrap(&mut self,wrap_mode: WrapMode) {
        self.command_buffer.push_back(FrameCommand::SetTextureWrap(wrap_mode));
    }

    /* Single Fire Draw Commands */
    
    pub fn draw_color(&mut self,parameters: PositionColor) {
        self.command_buffer.push_back(FrameCommand::DrawColor(parameters));
    }

    pub fn draw_frame(&mut self,frame: FinishedFrame,parameters: PositionUV) {
        self.command_buffer.push_back(FrameCommand::DrawFrame(frame,parameters));
    }

    pub fn draw_frame_colored(&mut self,frame: FinishedFrame,parameters: PositionUVColor) {
        self.command_buffer.push_back(FrameCommand::DrawFrameColored(frame,parameters));
    }
    
    /* Set Based Draw Commands  */

    pub fn draw_color_set(&mut self,parameters: Vec<PositionColor>) {
        self.command_buffer.push_back(FrameCommand::DrawColorSet(parameters));
    }

    pub fn draw_frame_set(&mut self,frame: FinishedFrame,parameters: Vec<PositionUV>) {
        self.command_buffer.push_back(FrameCommand::DrawFrameSet(frame,parameters));
    }

    pub fn draw_frame_colored_set(&mut self,frame: FinishedFrame,parameters: Vec<PositionUVColor>) {
        self.command_buffer.push_back(FrameCommand::DrawFrameColoredSet(frame,parameters));
    }

    /* Output & Interop */

    pub fn finish(&mut self,frame_binder: &mut FrameBinder,wgpu_interface: &impl WGPUInterface) -> FinishedFrame {
        if self.command_buffer.is_empty() {
            log::warn!("Frame command buffer is empty!");
        }
        let size = self.size();
        let frame = frame_binder.render_frame(&self,wgpu_interface);
        self.command_buffer.clear();
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
