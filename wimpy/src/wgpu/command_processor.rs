use wgpu::{
    BindGroup,
    RenderPass
};

use crate::wgpu::{
    FilterMode,
    FrameCommand,
    WrapMode,
    constants::BindGroupIndices,
    frame_cache::FrameCacheLookup,
    texture_container::SamplerMode,
    frame_cache::FrameCacheReference,
    double_buffer::DoubleBuffer,
    shader_definitions::QuadInstance
};

pub struct CommandProcessor<'a,TFrameCacheLookup> {
    needs_sampler_update: bool,
    filter_mode: FilterMode,
    wrap_mode: WrapMode,
    current_sampling_frame: FrameCacheReference,
    frame_cache: &'a TFrameCacheLookup,
}

enum CommandReturnFlow<'a> {
    Skip,
    Proceed(SamplerStatus<'a>)
}

enum SamplerStatus<'a> {
    Unchanged,
    UpdateNeeded(&'a BindGroup)
}

impl<'a,TFrameCacheLookup> CommandProcessor<'a,TFrameCacheLookup>
where
    TFrameCacheLookup: FrameCacheLookup
{
    fn update_sampler(&mut self,reference: FrameCacheReference) -> CommandReturnFlow {
        if !self.needs_sampler_update && self.current_sampling_frame == reference {
            return CommandReturnFlow::Proceed(SamplerStatus::Unchanged);
        }
        let sampler_bind_group = match self.frame_cache.get_texture_container(reference) {
            Ok(texture_container) => texture_container.get_bind_group(SamplerMode::get_mode(self.filter_mode,self.wrap_mode)),
            Err(error) => {
                log::error!("Unable to get sampler from texture container; the texture container cannot be found: {:?}",error);
                return CommandReturnFlow::Skip;
            }
        };
        self.needs_sampler_update = false;
        self.current_sampling_frame = reference;
        return CommandReturnFlow::Proceed(SamplerStatus::UpdateNeeded(sampler_bind_group));
    }

    fn execute(
        &mut self,
        instance_buffer: &'a mut DoubleBuffer<QuadInstance>,
        render_pass: &mut RenderPass,
        commands: &[FrameCommand]
    ) {
        for command in commands {
            match command {

                FrameCommand::DrawFrame(reference,draw_data) => match self.update_sampler(*reference) {
                    CommandReturnFlow::Proceed(sampler_status) => {
                        if let SamplerStatus::UpdateNeeded(bind_group) = sampler_status {
                            render_pass.set_bind_group(BindGroupIndices::TEXTURE,bind_group,&[]);
                        }
                        instance_buffer.write_quad(render_pass,&draw_data);
                    },
                    CommandReturnFlow::Skip => continue,
                },

                FrameCommand::DrawFrameSet(reference,draw_data) => match self.update_sampler(*reference) {
                    CommandReturnFlow::Proceed(sampler_status) => {
                        if let SamplerStatus::UpdateNeeded(bind_group) = sampler_status {
                            render_pass.set_bind_group(BindGroupIndices::TEXTURE,bind_group,&[]);
                        }
                        instance_buffer.write_quad_set(render_pass,&draw_data);
                    },
                    CommandReturnFlow::Skip => continue,
                },

                FrameCommand::SetTextureFilter(value) => {
                    let value = *value;
                    if self.filter_mode != value {
                        self.filter_mode = value;
                        self.needs_sampler_update = true;
                    }
                },

                FrameCommand::SetTextureWrap(value) => {
                    let value = *value;
                    if self.wrap_mode != value {
                        self.wrap_mode = value;
                        self.needs_sampler_update = true;
                    }
                },

            }
        }
    }
}

pub fn process_frame_commands<TFrameCacheLookup>(frame_cache: &TFrameCacheLookup,instance_buffer: &mut DoubleBuffer<QuadInstance>,render_pass: &mut RenderPass,commands: &[FrameCommand])
where
    TFrameCacheLookup: FrameCacheLookup
{
    let mut processor = CommandProcessor {
        needs_sampler_update: true,
        filter_mode: FilterMode::Nearest,
        wrap_mode: WrapMode::Clamp,
        current_sampling_frame: Default::default(),
        frame_cache,     
    };
    processor.execute(instance_buffer,render_pass,commands);
}
