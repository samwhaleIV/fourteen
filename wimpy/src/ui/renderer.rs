use crate::{
    app::VirtualDevice, shared::{
        Area,
        Layout
    }, ui::{
        NodeOutputBuilder,
        UINodeState
    }, wgpu::{
        Frame,
        GraphicsContext
    }
};

enum UIRendererCommand {
    OpenFrame,
    CloseFrame(Area),
    Draw(UINodeState<Area>)
}

struct UIRenderer {
    command_buffer: Vec<UIRendererCommand>,
}

impl NodeOutputBuilder<UINodeState<Layout>,UINodeState<Area>> for UIRenderer {
    fn start(&mut self) {
        self.command_buffer.clear();
    }

    fn start_branch(&mut self,input: &UINodeState<Layout>,branch_cache: &UINodeState<Area>) {
        todo!()
    }

    fn end_branch(&mut self,input: &UINodeState<Layout>,branch_cache: &UINodeState<Area>) {
        todo!()
    }

    fn next(&mut self,input: &UINodeState<Layout>,parent_cache: &UINodeState<Area>) -> UINodeState<Area> {
        todo!()
    }
}

impl UIRenderer {
    pub fn render(graphics_context: GraphicsContext<VirtualDevice>) {
        todo!();
    }
}
