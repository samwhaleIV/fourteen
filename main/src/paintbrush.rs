use std::{collections::VecDeque, ops::Range};

use wgpu::{RenderPass, TextureView};
use crate::graphics::{self, Graphics};

type RenderInstructionQueue = VecDeque<RenderInstruction>;

pub struct PaintBrush {
    render_pass_mode: RenderPassMode,
    instruction_queue: RenderInstructionQueue
}

struct DrawPrimitivesData {
    vertices: Range<u32>,
    instances: Range<u32>
}

enum RenderInstruction {
    DrawPrimitives(DrawPrimitivesData),
    SetColor(u8,u8,u8)
}

enum RenderPassMode {
    Basic,
    Custom
}

pub fn create_paint_brush() -> PaintBrush {
    return PaintBrush {
        render_pass_mode: RenderPassMode::Basic,
        instruction_queue: VecDeque::new()
    }
}

impl PaintBrush {
    pub fn draw_primitives(&mut self,vertices: Range<u32>,instances: Range<u32>) {
        self.instruction_queue.push_back(RenderInstruction::DrawPrimitives(DrawPrimitivesData{vertices,instances}));
    }

    pub fn render(&mut self,graphics: &Graphics,render_target: &TextureView) {

        if self.instruction_queue.is_empty() {
            log::warn!("Can't render because rendering instruction queue is empty.");
            return;
        }

        let mut encoder = graphics.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder")
        });

        {
            let mut render_pass = match self.render_pass_mode {
                RenderPassMode::Basic => graphics::get_basic_render_pass(&mut encoder,&render_target),
                RenderPassMode::Custom => todo!("Not implemented.")
            };

            render_pass.set_pipeline(&graphics.render_pipeline);

            for instruction in self.instruction_queue.iter() {
                execute_instruction(&mut render_pass,instruction);
            }
            self.instruction_queue.clear();
        }

        graphics.queue.submit(std::iter::once(encoder.finish()));
    }
}

fn execute_instruction(render_pass: &mut RenderPass,instruction: &RenderInstruction) {
    match instruction {
        RenderInstruction::DrawPrimitives(data) => {
            render_pass.draw(data.vertices.clone(),data.instances.clone());
        },
        _ => {}
    }
}

