use std::ops::Range;

use wgpu::RenderPass;

use crate::wgpu::{DrawData, constants::INDEX_BUFFER_SIZE, double_buffer::DoubleBuffer, shader_definitions::{CameraUniform, QuadInstance}};

pub struct DoubleBufferSet {
    pub instances: DoubleBuffer<QuadInstance>,
    pub uniforms: DoubleBuffer<CameraUniform>,
}

impl DoubleBufferSet {
    pub fn reset_all(&mut self) {
        self.instances.reset();
        self.uniforms.reset();
    }
}

impl DoubleBuffer<QuadInstance> {
    pub fn write_quad(&mut self,render_pass: &mut RenderPass,draw_data: &DrawData) {
        let range = self.push_convert(draw_data.into());
        render_pass.draw_indexed(0..INDEX_BUFFER_SIZE,0,downcast_range(range));
    }
    pub fn write_quad_set(&mut self,render_pass: &mut RenderPass,draw_data: &[DrawData]) {
        let range = self.push_convert_all(draw_data);
        render_pass.draw_indexed(0..INDEX_BUFFER_SIZE,0,downcast_range(range));
    }
}

const fn downcast_range(value: Range<usize>) -> Range<u32> {
    return Range {
        start: value.start as u32,
        end: value.end as u32,
    };
}
