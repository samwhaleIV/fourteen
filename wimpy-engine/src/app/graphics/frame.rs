use super::prelude::*;

#[derive(Copy,Clone)]
pub struct RestrictedSize {
    pub input: (u32,u32),
    pub output: (u32,u32),
}

#[derive(Copy,Clone)]
pub struct CacheSize {
    pub input: (u32,u32),
    pub output: u32
}

pub struct DrawData2D {
    pub destination: WimpyArea,
    pub source: WimpyArea,
    pub color: WimpyColor,
    pub rotation: f32
}

pub struct DrawData3D {
    pub transform: Matrix4<f32>,
    pub diffuse_color: WimpyColor,
    pub lightmap_color: WimpyColor,
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

impl Default for DrawData3D {
    fn default() -> Self {
        Self {
            transform: get_identity_matrix(),
            diffuse_color: WimpyColor::WHITE,
            lightmap_color: WimpyColor::WHITE,
        }
    }
}

pub enum FrameCommand {
    DrawFrame {
        reference: FrameCacheReference,
        draw_data: DrawData2D
    },
    DrawModel {
        reference: ModelCacheReference,
        draw_data: DrawData3D
    },
    SetTextureFilter(FilterMode),
    SetTextureAddressing(AddressMode),
}

pub struct OutputFrame {
    size: (u32,u32),
    cache_reference: FrameCacheReference,
    command_buffer: Vec<FrameCommand>,
    clear_color: wgpu::Color
}

#[derive(Clone,Copy,Debug)]
pub struct TextureFrame {
    size: (u32,u32),
    cache_reference: FrameCacheReference,
}

impl TextureFrame {
    pub fn placeholder() -> Self {
        return Self {
            size: (0,0),
            cache_reference: Default::default(),
        }
    }
}

pub struct TempFrame {
    size: CacheSize,
    cache_reference: FrameCacheReference,
    command_buffer: Vec<FrameCommand>,
    clear_color: wgpu::Color,
}

pub struct LongLifeFrame {
    size: RestrictedSize,
    cache_reference: FrameCacheReference,
    command_buffer: Vec<FrameCommand>,  
}

pub trait FrameReference {
    fn get_cache_reference(&self) -> FrameCacheReference;

    /// The size of the frame as requested by the user.
    fn get_input_size(&self) -> (u32,u32);

    /// The size of the real texture this frame renders to.
    fn get_output_size(&self) -> (u32,u32);

    fn get_output_uv_size(&self) -> (f32,f32) {
        let input = self.get_input_size();
        let output = self.get_output_size();

        (
            input.0 as f32 / output.0 as f32,
            input.1 as f32 / output.1 as f32,
        )
    }
}

pub trait FrameRenderPass<TFrame: MutableFrame> {
    fn create(frame: TFrame) -> Self;

    fn get_frame(&self) -> &TFrame;
    fn get_frame_mut(&mut self) -> &mut TFrame;

    fn begin_render_pass(
        self,
        render_pass: &mut RenderPass,
        pipeline_view: &mut RenderPassView
    ) -> TFrame;

    fn set_texture_filter(&mut self,filter_mode: FilterMode) {
        self.get_frame_mut().push_command(
            FrameCommand::SetTextureFilter(filter_mode)
        );
    }

    fn set_texture_addressing(&mut self,address_mode: AddressMode) {
        self.get_frame_mut().push_command(
            FrameCommand::SetTextureAddressing(address_mode)
        );
    }

    fn size(&self) -> (u32,u32) {
        self.get_frame().get_input_size()
    }
}

pub trait MutableFrame: FrameReference {
    fn push_command(&mut self,frame_command: FrameCommand);
    fn clear_commands(&mut self);
    fn get_commands(&self) -> &[FrameCommand];
    fn get_clear_color(&self) -> Option<wgpu::Color>;
}

impl FrameReference for OutputFrame {
    fn get_cache_reference(&self) -> FrameCacheReference {
        self.cache_reference
    }

    fn get_input_size(&self) -> (u32,u32) {
        self.size
    }

    fn get_output_size(&self) -> (u32,u32) {
        self.size
    }
}

impl FrameReference for TextureFrame {
    fn get_cache_reference(&self) -> FrameCacheReference {
        self.cache_reference
    }

    fn get_input_size(&self) -> (u32,u32) {
        self.size
    }

    fn get_output_size(&self) -> (u32,u32) {
        self.size
    }
}

impl FrameReference for TempFrame {
    fn get_cache_reference(&self) -> FrameCacheReference {
        self.cache_reference
    }

    fn get_input_size(&self) -> (u32,u32) {
        self.size.input
    }

    fn get_output_size(&self) -> (u32,u32) {
        (self.size.output,self.size.output)
    }
}

impl FrameReference for LongLifeFrame {
    fn get_cache_reference(&self) -> FrameCacheReference {
        self.cache_reference
    }

    fn get_input_size(&self) -> (u32,u32) {
        self.size.input
    }

    fn get_output_size(&self) -> (u32,u32) {
        self.size.output
    }
}

impl MutableFrame for OutputFrame {
    fn push_command(&mut self,frame_command: FrameCommand) {
        self.command_buffer.push(frame_command);
    }
    
    fn get_commands(&self) -> &[FrameCommand] {
        &self.command_buffer
    }
    
    fn clear_commands(&mut self) {
        self.command_buffer.clear();
    }
    
    fn get_clear_color(&self) -> Option<wgpu::Color> {
        Some(self.clear_color)
    }
}

impl MutableFrame for TempFrame {
    fn push_command(&mut self,frame_command: FrameCommand) {
        self.command_buffer.push(frame_command);
    }
    
    fn get_commands(&self) -> &[FrameCommand] {
        &self.command_buffer
    }
    
    fn clear_commands(&mut self) {
        self.command_buffer.clear();
    }
    
    fn get_clear_color(&self) -> Option<wgpu::Color> {
        Some(self.clear_color)
    }
}

impl MutableFrame for LongLifeFrame {
    fn push_command(&mut self,frame_command: FrameCommand) {
        self.command_buffer.push(frame_command);
    }
    
    fn get_commands(&self) -> &[FrameCommand] {
        &self.command_buffer
    }
    
    fn clear_commands(&mut self) {
        self.command_buffer.clear();
    }
    
    fn get_clear_color(&self) -> Option<wgpu::Color> {
        None
    }
}

pub struct FrameFactory;

impl FrameFactory {

    pub fn create_output(
        size: (u32,u32),
        cache_reference: FrameCacheReference,
        command_buffer: Vec<FrameCommand>,
        clear_color: wgpu::Color,
    ) -> OutputFrame {
        OutputFrame {
            size,
            cache_reference,
            command_buffer,
            clear_color,
        }
    }

    pub fn create_texture(
        size: (u32,u32),
        cache_reference: FrameCacheReference,
    ) -> TextureFrame {
        TextureFrame {
            size,
            cache_reference,
        }
    }

    pub fn create_long_life(
        size: RestrictedSize,
        cache_reference: FrameCacheReference,
        command_buffer: Vec<FrameCommand>
    ) -> LongLifeFrame {
        LongLifeFrame {
            size,
            cache_reference,
            command_buffer,
        }
    }

    pub fn create_temp_frame(
        size: CacheSize,
        cache_reference: FrameCacheReference,
        command_buffer: Vec<FrameCommand>,
        clear_color: wgpu::Color,
    ) -> TempFrame {
        TempFrame {
            size,
            cache_reference,
            command_buffer,
            clear_color
        }
    }
}

pub trait ReclaimCommandBuffer {
    fn take_command_buffer(self) -> Vec<FrameCommand>;
}

impl ReclaimCommandBuffer for OutputFrame {
    fn take_command_buffer(self) -> Vec<FrameCommand> {
        self.command_buffer
    }
}

impl ReclaimCommandBuffer for TempFrame {
    fn take_command_buffer(self) -> Vec<FrameCommand> {
        self.command_buffer
    }
}

impl ReclaimCommandBuffer for LongLifeFrame {
    fn take_command_buffer(self) -> Vec<FrameCommand> {
        self.command_buffer
    }
}
