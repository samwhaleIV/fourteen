mod creation;
mod shader_definitions;
pub use shader_definitions::*;

use super::*;

pub struct Pipeline2D {
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    instance_buffer: DoubleBuffer<QuadInstance>,
}

pub struct FrameRenderPass2D<'a,TFrame> {
    context: RenderPassContext<'a>,
    render_pass: RenderPass<'a>,
    frame: TFrame,
    needs_sampler_update: bool,
    sampler_mode: SamplerMode,
    current_sampling_frame: FrameCacheReference,
}

pub struct DrawData2D {
    pub destination: WimpyArea,
    pub source: WimpyArea,
    pub color: WimpyColor,
    pub rotation: f32
}

impl Default for DrawData2D {
    fn default() -> Self {
        Self {
            destination: WimpyArea::default(),
            source: WimpyArea::default(),
            color: WimpyColor::WHITE,
            rotation: 0.0
        }
    }
}

impl Pipeline2D {
    pub const VERTEX_BUFFER_INDEX: u32 = 0;
    pub const INSTANCE_BUFFER_INDEX: u32 = 1;
    pub const INDEX_BUFFER_SIZE: u32 = 6;
}

impl PipelineController for Pipeline2D {
    fn write_dynamic_buffers(&mut self,queue: &Queue) {
        self.instance_buffer.write_out(queue);
    }
    fn reset_pipeline_state(&mut self) {
        self.instance_buffer.reset();
    }
}

impl<'a,TFrame> FrameRenderPass<'a,TFrame> for FrameRenderPass2D<'a,TFrame>
where 
    TFrame: MutableFrame
{
    fn create(
        frame: TFrame,
        mut render_pass: RenderPass<'a>,
        mut context: RenderPassContext<'a>
    ) -> Self {
        let pipeline_2d = context.get_2d_pipeline();
        render_pass.set_pipeline(&pipeline_2d.render_pipeline); 

        render_pass.set_index_buffer(
            pipeline_2d.index_buffer.slice(..),
            wgpu::IndexFormat::Uint32
        ); // Index Buffer

        render_pass.set_vertex_buffer(
            Pipeline2D::VERTEX_BUFFER_INDEX,
            pipeline_2d.vertex_buffer.slice(..)
        ); // Vertex Buffer

        render_pass.set_vertex_buffer(
            Pipeline2D::INSTANCE_BUFFER_INDEX,
            pipeline_2d.instance_buffer.get_output_buffer().slice(..)
        ); // Instance Buffer

        let transform = TransformUniform::create_ortho(frame.get_input_size());
        let uniform_buffer_range = context.get_shared_mut().get_uniform_buffer().push(transform);
        let dynamic_offset = uniform_buffer_range.start * UNIFORM_BUFFER_ALIGNMENT;

        render_pass.set_bind_group(
            UNIFORM_BIND_GROUP_INDEX,
            context.get_shared().get_uniform_bind_group(),
            &[dynamic_offset as u32]
        );

        let current_sampling_frame = context.textures.transparent_black.get_cache_reference();

        return Self {
            context,
            render_pass,
            frame,
            needs_sampler_update: true,
            filter_mode: FilterMode::Linear,
            address_mode: AddressMode::Repeat,
            current_sampling_frame: current_sampling_frame
        }
    }

    fn finish(
        self
    ) -> TFrame {
        return self.frame;
    }
}

impl<TFrame> FrameRenderPass2D<'_,TFrame> {
    fn update_sampler_if_needed(&mut self,reference: FrameCacheReference) {
        if !self.needs_sampler_update && self.current_sampling_frame == reference {
            return;
        }

        self.current_sampling_frame = reference;
        self.needs_sampler_update = false;

        match self.context.frame_cache.get(reference) {
            Ok(texture_container) => match texture_container.get_bind_group(self.filter_mode,self.address_mode) {
                Some(bind_group) => {
                    self.render_pass.set_bind_group(TEXTURE_BIND_GROUP_INDEX,bind_group,&[]);
                },
                None => {
                    log::warn!("Unable to get sampler ({:?},{:?}) from texture container.",self.filter_mode,self.address_mode);
                }
            },
            Err(error) => {
                log::warn!("Unable to get texture container for sampler; the texture container cannot be found: {:?}",error);
            }
        };
    }

    pub fn draw(&mut self,frame_reference: &impl FrameReference,draw_data: &[DrawData2D]) {
        let reference = frame_reference.get_cache_reference();
        self.update_sampler_if_needed(reference);

        let output_size = frame_reference.get_output_uv_size();

        let range = self.context.get_2d_pipeline_mut().instance_buffer.push_set(draw_data.iter().map(|value|{
            let area = value.destination.to_center_encoded();
            let source = value.source.multiply_2d(output_size);
            QuadInstance {
                position: [area.x,area.y],
                size: [area.width,area.height],
                uv_position: [source.x,source.y],
                uv_size: [source.width,source.height],
                color: value.color.decompose(),
                rotation: value.rotation
            }
        }));

        self.render_pass.draw_indexed(0..Pipeline2D::INDEX_BUFFER_SIZE,0,downcast_range(range));
    }

    pub fn set_sampler_mode(&mut self,sampler_mode: SamplerMode) {
        if self.sampler_mode != sampler_mode {
            self.sampler_mode = self.sampler_mode;
            self.needs_sampler_update = true;
        }
    }
}
