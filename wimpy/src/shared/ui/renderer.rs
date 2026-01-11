use crate::{
    app::VirtualDevice,
    shared::{
        Area,
        Layout
    },
    ui::{
        LocateTexture,
        NodeOutputBuilder,
        UINodeInput,
        UINodeOutput,
    },
    wgpu::{
        Frame,
        GraphicsContext
    }
};

enum UIRendererCommand {
    OpenFrame(Area),
    CloseFrame,
    Draw(UINodeOutput)
}

struct UIRenderer {
    command_buffer: Vec<UIRendererCommand>,
    frame_buffer: Vec<Frame>,
}

impl NodeOutputBuilder<UINodeInput,UINodeOutput> for UIRenderer {
    fn clear(&mut self) {
        self.command_buffer.clear();
        self.frame_buffer.clear();
    }

    fn start_branch(&mut self,input: &UINodeInput,branch_cache: &UINodeOutput) {
        if !input.clip_children || input.is_root {
            return;
        }
        self.command_buffer.push(UIRendererCommand::OpenFrame(branch_cache.layout));
    }

    fn end_branch(&mut self,input: &UINodeInput,_: &UINodeOutput) {
        if !input.clip_children || input.is_root {
            return;
        }
        self.command_buffer.push(UIRendererCommand::CloseFrame);
    }

    fn next(&mut self,input: &UINodeInput,parent_cache: &UINodeOutput) -> UINodeOutput {
        return UINodeOutput {
            layout: input.layout.compute(parent_cache.layout),
            uv: input.uv,
            rotation: input.rotation,
            color: input.color,
            texture: input.texture,    
        };
    }
}

impl UIRenderer {
    pub fn render(&self,texture_locator: impl LocateTexture<Frame>,graphics_context: GraphicsContext<VirtualDevice>) {
        // for command in &self.command_buffer {

        // }
    }
}
