use wgpu::{
    AddressMode, BindGroup, FilterMode, RenderPass
};

use crate::wgpu::{
    FrameCommand,
    constants::BindGroupIndices,
    frame_cache::FrameCacheLookup,
    frame_cache::FrameCacheReference,
    double_buffer::DoubleBuffer,
    shader_definitions::QuadInstance
};

pub struct CommandProcessor<'render_pass,TFrameCacheLookup> {
    needs_sampler_update: bool,
    filter_mode: FilterMode,
    address_mode: AddressMode,
    current_sampling_frame: FrameCacheReference,
    frame_cache: &'render_pass TFrameCacheLookup,
}

enum CommandReturnFlow<'command> {
    Skip,
    Proceed(SamplerStatus<'command>)
}

enum SamplerStatus<'command> {
    Unchanged,
    UpdateNeeded(&'command BindGroup)
}

impl<TFrameCacheLookup> CommandProcessor<'_,TFrameCacheLookup>
where
    TFrameCacheLookup: FrameCacheLookup
{
    fn update_sampler(&mut self,reference: FrameCacheReference) -> CommandReturnFlow<'_> {
        if !self.needs_sampler_update && self.current_sampling_frame == reference {
            return CommandReturnFlow::Proceed(SamplerStatus::Unchanged);
        }

        let sampler_bind_group = match self.frame_cache.get_texture_container(reference) {
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
        &mut self,
        instance_buffer: &mut DoubleBuffer<QuadInstance>,
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

                FrameCommand::SetTextureAddressing(value) => {
                    let value = *value;
                    if self.address_mode != value {
                        self.address_mode = value;
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
        address_mode: AddressMode::ClampToEdge,
        current_sampling_frame: Default::default(),
        frame_cache,     
    };
    processor.execute(instance_buffer,render_pass,commands);
}
