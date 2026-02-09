use wgpu::*;

use crate::{
    shared::*,
    wgpu::*
};

pub const DEFAULT_COMMAND_BUFFER_SIZE: usize = 32;

struct OutputBuilder {
    encoder: CommandEncoder,
    output_frame_reference: FrameCacheReference,
    output_frame_surface: SurfaceTexture,
}

pub struct GraphicsContext {
    graphics_provider: GraphicsProvider,
    pipelines: RenderPipelines,
    frame_cache: FrameCache,
    output_builder: Option<OutputBuilder>,
    command_buffer_pool: VecPool<FrameCommand,DEFAULT_COMMAND_BUFFER_SIZE>,
    fallback_texture: TextureFrame
}

struct FallbackTexture {
    data: [u8;Self::DATA_SIZE]
}

impl FallbackTexture {
    const COLOR_1: [u8;Self::BYTES_PER_PIXEL] = [182,0,205,255];
    const COLOR_2: [u8;Self::BYTES_PER_PIXEL] = [53,23,91,255];

    const SIZE: usize = 32;
    const GRID_DIVISION: usize = 4;
    const BYTES_PER_PIXEL: usize = 4;
    const PIXEL_COUNT: usize = Self::SIZE * Self::SIZE;
    const DATA_SIZE: usize = Self::PIXEL_COUNT * 4;

    fn get_color(x: usize,y: usize) -> [u8;Self::BYTES_PER_PIXEL] {
        let column = x / Self::GRID_DIVISION;
        let row = y / Self::GRID_DIVISION;

        let checker_pattern = (column + row) % 2 == 0;

        return match checker_pattern {
            true => Self::COLOR_1,
            false => Self::COLOR_2
        };
    }

    fn create() -> Self {
        let mut data: [u8;Self::DATA_SIZE] = [0;Self::DATA_SIZE];

        let mut i: usize = 0;
        while i < Self::PIXEL_COUNT {
            let x = i % Self::SIZE;
            let y = i / Self::SIZE;

            let color = Self::get_color(x,y);

            data[i * Self::BYTES_PER_PIXEL + 0] = color[0];
            data[i * Self::BYTES_PER_PIXEL + 1] = color[1];
            data[i * Self::BYTES_PER_PIXEL + 2] = color[2];
            data[i * Self::BYTES_PER_PIXEL + 3] = color[3];

            i += 1;
        }

        return Self {
            data
        }
    }
}

impl TextureData for FallbackTexture {
    fn write_to_queue(&self,parameters: &TextureDataWriteParameters) {
        parameters.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: parameters.texture,
                mip_level: parameters.mip_level,
                origin: parameters.origin,
                aspect: parameters.aspect,
            },
            &self.data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(Self::SIZE as u32 * 4), 
                rows_per_image: Some(Self::SIZE as u32),
            },
            parameters.texture_size,
        );
    }
    fn size(&self) -> (u32,u32) {
        return (Self::SIZE as u32,Self::SIZE as u32);
    }
}

impl GraphicsContext {
    pub fn get_graphics_provider(&self) -> &GraphicsProvider {
        return &self.graphics_provider;
    }
    pub fn get_graphics_provider_mut(&mut self) -> &mut GraphicsProvider {
        return &mut self.graphics_provider;
    }
    pub fn create<TConfig: GraphicsContextConfig>(graphics_provider: GraphicsProvider) -> Self {

        let render_pipelines = RenderPipelines::create::<TConfig>(&graphics_provider);

        let mut graphics_context = Self {
            graphics_provider,
            pipelines: render_pipelines,
            frame_cache: FrameCache::default(),
            command_buffer_pool: VecPool::new(),
            output_builder: None,
            fallback_texture: TextureFrame::get_fake()
        };

        let texture_data = FallbackTexture::create();
        if let Ok(texture_frame) = graphics_context.create_texture_frame(&texture_data) {
            graphics_context.fallback_texture = texture_frame;
        } else {
            log::error!("Could not create a computed fallback texture frame. The fallback texture is set to a fake frame cache reference.");
        }

        return graphics_context;
    }
}

pub trait GraphicsContextConfig {
    // These are in byte count
    const UNIFORM_BUFFER_SIZE: usize;
    const INSTANCE_BUFFER_SIZE_2D: usize;
    const MODEL_CACHE_VERTEX_BUFFER_SIZE: usize;
    const MODEL_CACHE_INDEX_BUFFER_SIZE: usize;
    const INSTANCE_BUFFER_SIZE_3D: usize;
}

pub trait GraphicsContextController {
    fn create_texture_frame(&mut self,texture_data: &impl TextureData) -> Result<TextureFrame,TextureContainerError>;

    fn begin_frame_pass<TFrame: MutableFrame,TRenderPass: FrameRenderPass<TFrame>>(&mut self,frame: TFrame) -> TRenderPass {
        TRenderPass::create(frame)
    }
    fn finish_frame_pass<TFrame: MutableFrame,TRenderPass: FrameRenderPass<TFrame>>(&mut self,frame_render_pass: TRenderPass) -> Result<TFrame,GraphicsContextError>;

    fn get_cache_safe_size(&self,size: (u32,u32)) -> CacheSize;
    fn ensure_frame_for_cache_size(&mut self,cache_size: CacheSize);

    fn get_temp_frame(&mut self,cache_size: CacheSize,clear_color: wgpu::Color) -> TempFrame;
    fn return_temp_frame(&mut self,frame: TempFrame) -> Result<(),GraphicsContextError>;

    fn get_long_life_frame(&mut self,size: (u32,u32)) -> LongLifeFrame;
    fn return_long_life_frame(&mut self,frame: LongLifeFrame) -> Result<(),GraphicsContextError>;

    fn get_fallback_texture_frame(&self) -> &TextureFrame;
}

#[derive(Debug)]
pub enum GraphicsContextError {
    PipelineAlreadyActive,
    PipelineNotActive,
    CantCreateOutputSurface(SurfaceError),
    FrameCacheError(CacheArenaError<u32,FrameCacheReference>),
}

pub trait GraphicsContextInternalController {
    fn create_output_frame(&mut self,clear_color: wgpu::Color) -> Result<OutputFrame,GraphicsContextError>;
    fn present_output_frame(&mut self,frame: OutputFrame) -> Result<(),GraphicsContextError>;
}

impl GraphicsContextInternalController for GraphicsContext {
    fn create_output_frame(&mut self,clear_color: wgpu::Color) -> Result<OutputFrame,GraphicsContextError> {
        if self.output_builder.is_some() {
            return Err(GraphicsContextError::PipelineAlreadyActive);
        }

        let surface = match self.graphics_provider.get_output_surface() {
            Ok(value) => value,
            Err(error) => return Err(GraphicsContextError::CantCreateOutputSurface(error)),
        };

        let size = (surface.texture.width(),surface.texture.height());

        let texture_container = TextureContainer::create_output(&surface,size);
        let cache_reference = self.frame_cache.insert_keyless(texture_container);

        self.output_builder = Some(OutputBuilder {
            output_frame_reference: cache_reference,
            output_frame_surface: surface,
            encoder: self.graphics_provider.get_device().create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder")
            })
        });

        return Ok(FrameFactory::create_output(
            size,
            cache_reference,
            self.command_buffer_pool.take_item(),
            clear_color
        ));
    }

    fn present_output_frame(&mut self,frame: OutputFrame) -> Result<(),GraphicsContextError> {
        let Some(output_builder) = self.output_builder.take() else { //see if there's ANY way to avoid .take() here
            return Err(GraphicsContextError::PipelineNotActive);
        };
        let queue = self.graphics_provider.get_queue();
        
        // Investigate: only finalize the pipelines that were used during this output builder's life (or let the pipelines no-op on their own?)
        self.pipelines.pipeline_2d.write_buffers(queue);
        self.pipelines.pipeline_3d.write_buffers(queue);
        
        // We always write the shared buffers
        self.pipelines.shared.write_buffers(queue);

        queue.submit(std::iter::once(output_builder.encoder.finish()));
        if let Err(error) = self.frame_cache.remove(output_builder.output_frame_reference) {
            log::warn!("Output frame was not present in the frame cache: {:?}",error);
        };
        output_builder.output_frame_surface.present();
        
        self.pipelines.pipeline_2d.reset_buffers();
        self.pipelines.pipeline_3d.reset_buffers();

        self.pipelines.shared.reset_buffers();

        self.command_buffer_pool.return_item(frame.take_command_buffer());
        return Ok(());
    }
}

impl GraphicsContextController for GraphicsContext {
    fn create_texture_frame(&mut self,texture_data: &impl TextureData) -> Result<TextureFrame,TextureContainerError> {
        let texture_container = TextureContainer::from_image(
            &self.graphics_provider,
            &self.pipelines.shared.texture_layout,
            texture_data
        )?;
        return Ok(FrameFactory::create_texture(
            texture_container.size(),
            self.frame_cache.insert_keyless(texture_container)
        ));
    }

    fn get_temp_frame(&mut self,cache_size: CacheSize,clear_color: wgpu::Color) -> TempFrame {
        let size = cache_size.output;
        let cache_reference = match self.frame_cache.start_lease(size) {
            Ok(value) => value, 
            Err(error) => {
                log::warn!("Graphics context creating a new temp frame. Reason: {:?}",error);
                self.frame_cache.insert_with_lease(size,TextureContainer::create_mutable(
                    &self.graphics_provider,
                    &self.pipelines.shared.texture_layout,
                    (size,size)
                ))
            },
        };
        return FrameFactory::create_temp_frame(
            cache_size,
            cache_reference,
            self.command_buffer_pool.take_item(),
            clear_color
        );
    }
    
    fn return_temp_frame(&mut self,frame: TempFrame) -> Result<(),GraphicsContextError> {
        let cache_reference = frame.get_cache_reference();

        self.command_buffer_pool.return_item(frame.take_command_buffer());

        if let Err(error) = self.frame_cache.end_lease(cache_reference) {
            return Err(GraphicsContextError::FrameCacheError(error));
        }

        Ok(())
    }

    fn get_long_life_frame(&mut self,size: (u32,u32)) -> LongLifeFrame {
        let output = self.graphics_provider.get_safe_texture_size(size);
        return FrameFactory::create_long_life(
            RestrictedSize {
                input: size,
                output
            },
            self.frame_cache.insert_keyless(TextureContainer::create_mutable(
                &self.graphics_provider,
                &self.pipelines.shared.texture_layout,
                output
            )),
            Vec::with_capacity(DEFAULT_COMMAND_BUFFER_SIZE)
        );
    }

    fn return_long_life_frame(&mut self,frame: LongLifeFrame) -> Result<(),GraphicsContextError> {
        let cache_reference = frame.get_cache_reference();

        self.command_buffer_pool.return_item(frame.take_command_buffer());

        if let Err(error) = self.frame_cache.remove(cache_reference) {
            return Err(GraphicsContextError::FrameCacheError(error));
        }

        Ok(())
    }

    fn get_cache_safe_size(&self,size: (u32,u32)) -> CacheSize {
        let output = self.graphics_provider.get_safe_texture_power_of_two(match size.0.max(size.1).checked_next_power_of_two() {
            Some(value) => value,
            None => u32::MAX,
        });
        CacheSize {
            input: size,
            output
        }
    }

    fn ensure_frame_for_cache_size(&mut self,cache_size: CacheSize) {
        let size = cache_size.output;
        if self.frame_cache.has_available_items(size) {
            return;
        }
        self.frame_cache.insert(size,TextureContainer::create_mutable(
            &self.graphics_provider,
            &self.pipelines.shared.texture_layout,
            (size,size)
        ));
    }
    
    fn finish_frame_pass<TFrame: MutableFrame,TRenderPass: FrameRenderPass<TFrame>>(&mut self,mut frame_render_pass: TRenderPass) -> Result<TFrame,GraphicsContextError> {
        let frame = frame_render_pass.get_frame_mut();

        let Some(frame_builder) = &mut self.output_builder else {
            frame.clear_commands();
            return Err(GraphicsContextError::PipelineNotActive);
        };

        let texture_view = match self.frame_cache.get(frame.get_cache_reference()) {
            Ok(value) => value.get_view(),
            Err(error) => {
                frame.clear_commands();
                return Err(GraphicsContextError::FrameCacheError(error));
            }
        };

        let mut render_pass = frame_builder.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: match frame.get_clear_color() {
                        Some(color) => wgpu::LoadOp::Clear(color),
                        None => wgpu::LoadOp::Load,
                    },
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        let mut frame = frame_render_pass.begin_hardware_pass(&mut render_pass,&mut self.pipelines);

        process_frame_commands(
            frame.get_commands(),
            &mut render_pass,
            &mut self.pipelines,
            &self.frame_cache
        );

        frame.clear_commands();
        return Ok(frame);
    }
    
    fn get_fallback_texture_frame(&self) -> &TextureFrame {
        return &self.fallback_texture;
    }
}
