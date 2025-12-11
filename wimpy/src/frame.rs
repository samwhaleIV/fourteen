#![allow(dead_code,unused_variables)]

use std::collections::VecDeque;
use crate::{area::Area, color::Color, frame_binder::{FrameBinder, WGPUInterface}, texture::Texture};

pub struct Frame {
    width: u32,
    height: u32,
    command_buffer: VecDeque<FrameCommand>
}

pub struct FinishedFrame {
    width: u32,
    height: u32,
    reference: generational_arena::Index
}

pub enum FrameCommand {
    SetSamplerSettings(SamplerSettings),

    DrawColor(Vec<PositionColorParameters>),

    DrawTexture((Texture,Vec<PositionUVParameters>)),
    DrawTextureColored((Texture,Vec<PositionUVColorParameters>)),

    DrawFrame((Frame,Vec<PositionUVParameters>)),
    DrawFrameColored((Frame,Vec<PositionUVColorParameters>)),
}

pub enum SamplerAddressMode {
    Clamp,
    Repeat,
    MirrorRepeat
}

pub enum SamplerFilterMode {
    Nearest,
    Linear,
}

pub enum SamplerMirroringMode {
    None,
    Horizontal,
    Vertical,
    Both
}

pub struct SamplerSettings {
    address_mode: SamplerAddressMode,
    filter_mode: SamplerFilterMode,
    mirror_mode: SamplerMirroringMode
}

pub struct PositionColorParameters {
    position: Area,
    color: Color
}

pub struct PositionUVParameters {
    position: Area,
    uv: Area,
}

pub struct PositionUVColorParameters {
    position: Area,
    uv: Area,
    color: Color
}

impl Frame {
    pub fn width(&self) -> u32 {
        return self.width;
    }
    pub fn height(&self) -> u32 {
        return self.height;
    }

    pub fn size(&self) -> (u32,u32) {
        return (self.width,self.height);
    }

    pub fn set_sampler_settings(&mut self,sampler_settings: SamplerSettings) {
        self.command_buffer.push_back(FrameCommand::SetSamplerSettings(sampler_settings));
    }

    pub fn draw_color(&mut self,parameters: Vec<PositionColorParameters>) {
        self.command_buffer.push_back(FrameCommand::DrawColor(parameters));
    }
    
    pub fn draw_texture(&mut self,texture: Texture,parameters: Vec<PositionUVParameters>) {
        self.command_buffer.push_back(FrameCommand::DrawTexture((texture,parameters)));
    }

    pub fn draw_texture_colored(&mut self,texture: Texture,parameters: Vec<PositionUVColorParameters>) {
        self.command_buffer.push_back(FrameCommand::DrawTextureColored((textureparameters));
    }

    pub fn draw_frame(&mut self,frame: FinishedFrame,parameters: Vec<PositionUVParameters>) {
        self.command_buffer.push_back(FrameCommand::DrawFrame(parameters));
    }

    pub fn draw_frame_colored(&mut self,frame: FinishedFrame,parameters: Vec<PositionUVColorParameters>) {
        self.command_buffer.push_back(FrameCommand::DrawFrameColored(parameters));
    }

    pub fn render(&mut self,frame_binder: &mut FrameBinder,wgpu_interface: &impl WGPUInterface) -> FinishedFrame {
        if self.command_buffer.is_empty() {
            log::warn!("Frame command buffer is empty!");
        }
        let frame = frame_binder.render_frame(&self, wgpu_interface);
        self.command_buffer.clear();
        return frame;
    }
}

pub fn create_frame(width: u32,height: u32) -> Frame {
    return Frame {
        width,
        height,
        command_buffer: VecDeque::default()
    };
}
