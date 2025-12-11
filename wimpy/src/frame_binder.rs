use std::{collections::VecDeque, sync::Arc};

use generational_arena::Arena;

use crate::frame::{FinishedFrame, Frame, FrameCommand};


/* Figure out later which type this binds to in WGPU */
type InternalFinishedFrame = u32;

pub struct FrameBinder {
    finished_frames: Arena<InternalFinishedFrame>
}

impl FrameBinder {
    pub fn render_frame(&mut self,frame: &Frame,wgpu_interface: &impl WGPUInterface) -> FinishedFrame {
        let (width,height) = frame.size();

        

    }
}

pub trait WGPUInterface {

}
