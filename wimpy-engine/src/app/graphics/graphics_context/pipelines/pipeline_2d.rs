mod creation;
mod shader_definitions;
pub use shader_definitions::*;

use crate::collections::VecPool;
use super::*;

pub struct Pipeline2D {
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    instance_buffer: DoubleBuffer<QuadInstance>,
    command_buffer_pool: VecPool<Pipeline2DCommand,DEFAULT_COMMAND_BUFFER_SIZE>
}

pub struct FrameRenderPass2D<TFrame> {
    frame: TFrame,
    command_buffer: Vec<Pipeline2DCommand>
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

    pub fn draw_quad(&mut self,render_pass: &mut RenderPass,draw_data: &DrawData2D) {
        let range = self.instance_buffer.push_convert(draw_data.into());
        render_pass.draw_indexed(0..Self::INDEX_BUFFER_SIZE,0,downcast_range(range));
    }

    pub fn draw_quad_set(&mut self,render_pass: &mut RenderPass,draw_data: &[DrawData2D]) {
        let range = self.instance_buffer.push_convert_all(draw_data);
        render_pass.draw_indexed(0..Self::INDEX_BUFFER_SIZE,0,downcast_range(range));
    }
}

impl PipelineController for Pipeline2D {
    fn write_dynamic_buffers(&mut self,queue: &Queue) {
        self.instance_buffer.write_out(queue);
    }
    fn reset_pipeline_state(&mut self) {
        self.instance_buffer.reset();
    }
}

impl<TFrame> FrameRenderPass<TFrame> for FrameRenderPass2D<TFrame>
where 
    TFrame: MutableFrame
{
    fn create(frame: TFrame,render_pass_view: &mut RenderPassView) -> Self {
        let command_buffer = render_pass_view.get_2d_pipeline_mut().command_buffer_pool.take_item();
        return Self {
            frame,
            command_buffer,
        }
    }

    fn begin_render_pass(self,render_pass: &mut RenderPass,render_pass_view: &mut RenderPassView) -> TFrame {
        let pipeline_2d = render_pass_view.get_2d_pipeline();

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


        let shared_pipeline = render_pass_view.get_shared_pipeline_mut();

        let transform = MatrixTransformUniform::create_ortho(self.size());
        let uniform_buffer_range = shared_pipeline.get_uniform_buffer().push(transform);
        let dynamic_offset = uniform_buffer_range.start * UNIFORM_BUFFER_ALIGNMENT;

        render_pass.set_bind_group(
            UNIFORM_BIND_GROUP_INDEX,
            shared_pipeline.get_uniform_bind_group(),
            &[dynamic_offset as u32]
        );

        let command_processor = CommandProcessor {
            needs_sampler_update: true,
            filter_mode: FilterMode::Nearest,
            address_mode: AddressMode::ClampToEdge,
            current_sampling_frame: Default::default(),
            render_pass_view
        };
        command_processor.execute(&self.command_buffer,render_pass);

        render_pass_view.get_2d_pipeline_mut().command_buffer_pool.return_item(self.command_buffer);

        self.frame
    }

    fn get_frame(&self) -> &TFrame {
        return &self.frame;
    }
    
    fn get_frame_mut(&mut self) -> &mut TFrame {
        return &mut self.frame;
    }
}

pub struct CommandProcessor<'a,'render_pass> {
    needs_sampler_update: bool,
    filter_mode: FilterMode,
    address_mode: AddressMode,
    current_sampling_frame: FrameCacheReference,
    render_pass_view: &'a mut RenderPassView<'render_pass>
}

enum CommandReturnFlow<'command> {
    Skip,
    Proceed(SamplerStatus<'command>)
}

enum SamplerStatus<'command> {
    Unchanged,
    UpdateNeeded(&'command BindGroup)
}

enum Pipeline2DCommand {
    Draw {
        reference: FrameCacheReference,
        draw_data: DrawData2D
    },
    SetTextureFilter(FilterMode),
    SetTextureAddressing(AddressMode),
}

impl CommandProcessor<'_,'_> {
    fn update_sampler(&mut self,reference: FrameCacheReference) -> CommandReturnFlow<'_> {
        if !self.needs_sampler_update && self.current_sampling_frame == reference {
            return CommandReturnFlow::Proceed(SamplerStatus::Unchanged);
        }

        let sampler_bind_group = match self.render_pass_view.frame_cache.get(reference) {
            Ok(texture_container) => match texture_container.get_bind_group(self.filter_mode,self.address_mode) {
                Some(value) => value,
                None => {
                    log::warn!("Unable to get sampler ({:?},{:?}) from texture container.",self.filter_mode,self.address_mode);
                    return CommandReturnFlow::Skip;
                }
            },
            Err(error) => {
                log::warn!("Unable to get texture container for sampler; the texture container cannot be found: {:?}",error);
                return CommandReturnFlow::Skip;
            }
        };
        self.needs_sampler_update = false;
        self.current_sampling_frame = reference;
        return CommandReturnFlow::Proceed(SamplerStatus::UpdateNeeded(sampler_bind_group));
    }

    fn execute(
        mut self,
        commands: &[Pipeline2DCommand],
        render_pass: &mut RenderPass,
    ) {
        for command in commands {
            match command {
                Pipeline2DCommand::Draw {
                    reference,
                    draw_data
                } => match self.update_sampler(*reference) {
                    CommandReturnFlow::Proceed(sampler_status) => {
                        if let SamplerStatus::UpdateNeeded(bind_group) = sampler_status {
                            render_pass.set_bind_group(TEXTURE_BIND_GROUP_INDEX,bind_group,&[]);
                        }
                        self.render_pass_view.get_2d_pipeline_mut().draw_quad(render_pass,draw_data);
                    },
                    CommandReturnFlow::Skip => continue,
                },
                Pipeline2DCommand::SetTextureFilter(value) => {
                    let value = *value;
                    if self.filter_mode != value {
                        self.filter_mode = value;
                        self.needs_sampler_update = true;
                    }
                },
                Pipeline2DCommand::SetTextureAddressing(value) => {
                    let value = *value;
                    if self.address_mode != value {
                        self.address_mode = value;
                        self.needs_sampler_update = true;
                    }
                }
            }
        }
    }
}

impl<TFrame> FrameRenderPass2D<TFrame>
where 
    TFrame: MutableFrame
{
    pub fn draw(&mut self,frame_reference: &impl FrameReference,draw_data: DrawData2D) {
        self.command_buffer.push(
            Pipeline2DCommand::Draw {
                reference: frame_reference.get_cache_reference(),
                draw_data: DrawData2D {
                    destination: draw_data.destination,
                    source: draw_data.source.multiply_2d(frame_reference.get_output_uv_size()),
                    color: draw_data.color,
                    rotation: draw_data.rotation
                }
            }
        );
    }
    pub fn set_texture_filter(&mut self,filter_mode: FilterMode) {
        self.command_buffer.push(
            Pipeline2DCommand::SetTextureFilter(filter_mode)
        );
    }

    pub fn set_texture_addressing(&mut self,address_mode: AddressMode) {
        self.command_buffer.push(
            Pipeline2DCommand::SetTextureAddressing(address_mode)
        );
    }
}
