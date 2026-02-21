use wgpu::*;
use wgpu::util::{BufferInitDescriptor,DeviceExt};
use std::ops::Range;
use bytemuck::{Pod,Zeroable};
use crate::app::graphics::{*,constants::*};
use crate::shared::*;
use super::core::*;

pub struct TextPipeline {
    pipelines: PipelineVariants,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    instance_buffer: DoubleBuffer<GlyphInstance>,
}

pub trait FontDefinition {
    fn get_texture(textures: &EngineTextures) -> TextureFrame;
    fn get_glyph(character: char) -> GlyphArea;

    const LINE_HEIGHT: f32;
    const LETTER_SPACING: f32;
    const WORD_SPACING: f32;

    fn get_word_spacing(scale: f32) -> f32 {
        (Self::WORD_SPACING * scale).round().max(1.0)
    }

    fn get_letter_spacing(scale: f32) -> f32 {
        (Self::LETTER_SPACING * scale).round().max(1.0)
    }

    fn get_line_height(scale: f32) -> f32 {
        (Self::LINE_HEIGHT * scale).round().max(1.0)
    }
}

#[derive(Default)]
pub struct GlyphArea {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub y_offset: i16
}

impl TextPipeline {
    pub const VERTEX_BUFFER_INDEX: u32 = 0;
    pub const INSTANCE_BUFFER_INDEX: u32 = 1;
    pub const INDEX_BUFFER_SIZE: u32 = 6;

    pub fn create<TConfig>(
        graphics_provider: &GraphicsProvider,
        texture_layout: &BindGroupLayout,
        uniform_layout: &BindGroupLayout,
    ) -> Self
    where
        TConfig: GraphicsContextConfig
    {
        let device = graphics_provider.get_device();

        let shader = &device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Text Pipeline Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/text_pipeline.wgsl").into())
        });

        let render_pipeline_layout = &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Text Pipeline Render Layout"),
            bind_group_layouts: &[
                texture_layout,
                uniform_layout,
            ],
            push_constant_ranges: &[]
        });

        let pipeline_creator = PipelineCreator {
            graphics_provider,
            render_pipeline_layout,
            shader,
            vertex_buffer_layout: &[
                GlyphVertex::get_buffer_layout(),
                GlyphInstance::get_buffer_layout()
            ],
            primitive_state: &wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false
            },
            label: "Text Pipeline",
        };
        let pipelines = pipeline_creator.create_variants();

        let vertices = [  
            GlyphVertex { position: [-0.5,-0.5] },
            GlyphVertex { position: [-0.5, 0.5] },
            GlyphVertex { position: [0.5,-0.5] },
            GlyphVertex { position: [0.5, 0.5] }
        ];

        let indices: [u32;Self::INDEX_BUFFER_SIZE as usize] = [
            0,1,2,
            2,1,3
        ];

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor{
            label: Some("Text Pipeline Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX
        });

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor{
            label: Some("Text Pipeline Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX
        });

        let instance_buffer = DoubleBuffer::new(
            device.create_buffer(&BufferDescriptor{
                label: Some("Text Pipeline Instance Buffer"),
                size: TConfig::TEXT_PIPELINE_BUFFER_SIZE as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        );

        return Self {
            pipelines,
            vertex_buffer,
            index_buffer,
            instance_buffer,
        }
    }
}

pub struct PipelineTextPass<'a,'frame> {
    context: &'a mut RenderPassContext<'frame>,
    render_pass: &'a mut RenderPass<'frame>,
    tex: TextureFrame,
    uv_scalar: (f32,f32),
}

pub enum TextRenderBehavior {
    LTR,
    RTL,
    Centered,
    LineBreakingLTR {
        line_width: f32
    }
}

pub struct TextRenderConfig {
    pub position: (f32,f32),
    pub scale: f32,
    pub color: WimpyColor,
    pub line_height: f32,
    pub word_seperator: char,
    pub behavior: TextRenderBehavior
}

fn validate_scale(scale: f32) -> f32 {
    scale.round().max(1.0)
}

fn measure_word_width<FFont: FontDefinition>(word: &str,scale: f32,letter_spacing: f32) -> f32 {
    let mut width = 0.0_f32;
    for character in word.chars() {
        let glyph = FFont::get_glyph(character);
        width += glyph.width as f32 * scale + letter_spacing;
    }
    width -= letter_spacing;
    return width;
}

impl PipelineController for TextPipeline {
    fn write_dynamic_buffers(&mut self,queue: &Queue) {
        self.instance_buffer.write_out(queue);
    }
    fn reset_pipeline_state(&mut self) {
        self.instance_buffer.reset();
    }
}

impl<'a,'frame> PipelinePass<'a,'frame> for PipelineTextPass<'a,'frame> {
    fn create(
        frame: &'frame impl MutableFrame,
        render_pass: &'a mut RenderPass<'frame>,
        context: &'a mut RenderPassContext<'frame>
    ) -> Self {
        let text_pipeline = context.get_text_pipeline();

        render_pass.set_pipeline(text_pipeline.pipelines.select(frame));

        render_pass.set_index_buffer(
            text_pipeline.index_buffer.slice(..),
            wgpu::IndexFormat::Uint32
        );

        render_pass.set_vertex_buffer(
            TextPipeline::VERTEX_BUFFER_INDEX,
            text_pipeline.vertex_buffer.slice(..)
        );

        render_pass.set_vertex_buffer(
            TextPipeline::INSTANCE_BUFFER_INDEX,
            text_pipeline.instance_buffer.get_output_buffer().slice(..)
        );

        let transform = TransformUniform::create_ortho(frame.size());
        let uniform_buffer_range = context.get_shared_mut().get_uniform_buffer().push(transform);
        let dynamic_offset = uniform_buffer_range.start * UNIFORM_BUFFER_ALIGNMENT;

        render_pass.set_bind_group(
            UNIFORM_BIND_GROUP_INDEX,
            context.get_shared().get_uniform_bind_group(),
            &[dynamic_offset as u32]
        );

        let tex = context.textures.transparent_black.clone();

        return Self {
            uv_scalar: (1.0,1.0),
            tex,
            context,
            render_pass,
        }
    }
}

impl PipelineTextPass<'_,'_> {
    fn validate_texture<TFont: FontDefinition>(&mut self) -> bool {
        let current_texture = self.tex;
        let target_texture = TFont::get_texture(self.context.textures);
        let target_texture_ref = target_texture.get_ref();

        if current_texture.get_ref() != target_texture.get_ref() {
            match self.context.frame_cache.get(target_texture_ref) {
                Ok(texture_container) => self.context.set_texture_bind_group(&mut self.render_pass,&BindGroupCacheIdentity::SingleChannel {
                    ch_0: BindGroupChannelConfig {
                        mode: SamplerMode::NearestClamp,
                        texture: texture_container,
                    }
                }),
                Err(error) => {
                    log::warn!("Unable to get texture container for sampler; the texture container cannot be found: {:?}",error);
                    return false;
                }
            };
            self.tex = target_texture;
            let scale = self.tex.get_uv_scale();
            self.uv_scalar = (
                (1.0 / self.tex.width() as f32) * scale.0,
                (1.0 / self.tex.height() as f32) * scale.1,
            );
        }

        return true;
    }

    fn draw_glyph<TFont: FontDefinition>(&mut self,character: char,x: f32,mut y: f32,scale: f32,color: &WimpyColor) -> f32 {
        let glyph = TFont::get_glyph(character);
        if glyph.width == 0 {
            return 0.0;
        }
        y += glyph.y_offset as f32 * scale;

        let width = glyph.width as f32;
        let height = glyph.height as f32;

        let source = WimpyArea {
            x: (glyph.x as f32),
            y: glyph.y as f32,
            width,
            height,
        }.multiply_2d(self.uv_scalar);

        let destination = WimpyArea {
            x: x,
            y: y,
            width: width * scale,
            height: height * scale,
        }.to_center_encoded();

        let glyph_instance = GlyphInstance {
            position: [destination.x,destination.y],
            size: [destination.width,destination.height],
            uv_position: [source.x,source.y],
            uv_size: [source.width,source.height],
            color: color.decompose(),
        };

        self.context.pipelines.get_unique_mut().text_pipeline.instance_buffer.push(glyph_instance);

        return destination.width;
    }

    fn draw_text_line_breaking_ltr<TFont: FontDefinition>(&mut self,text: &str,config: TextRenderConfig,max_width: f32) {
        let scale = validate_scale(config.scale);
        let word_spacing = TFont::get_word_spacing(scale);
        let letter_spacing = TFont::get_letter_spacing(scale);
        let line_height = TFont::get_line_height(scale * config.line_height);

        let x_start = config.position.0.round();
        let max_x = x_start + max_width;

        let mut x = x_start;
        let mut y = config.position.1.round();

        let color = &config.color;
        for word in text.split(config.word_seperator) {
            if x + measure_word_width::<TFont>(word,scale,letter_spacing) > max_x {
                x = x_start;
                y += line_height;
            }
            for character in word.chars() {
                x += self.draw_glyph::<TFont>(character,x,y,scale,color) + letter_spacing;
            }
            x += word_spacing - letter_spacing;
        }
    }

    fn draw_text_ltr<TFont: FontDefinition>(&mut self,text: &str,config: TextRenderConfig) {
        let scale = validate_scale(config.scale);
        let word_spacing = TFont::get_word_spacing(scale);
        let letter_spacing = TFont::get_letter_spacing(scale);

        let mut x = config.position.0.round();
        let y = config.position.1.round();

        let color = &config.color;
        for word in text.split(config.word_seperator) {
            for character in word.chars() {
                x += self.draw_glyph::<TFont>(character,x,y,scale,color) + letter_spacing;
            }
            x += word_spacing - letter_spacing;
        }
    }

    fn draw_text_rtl<TFont: FontDefinition>(&mut self,text: &str,config: TextRenderConfig) {
        let scale = validate_scale(config.scale);
        let word_spacing = TFont::get_word_spacing(scale);
        let letter_spacing = TFont::get_letter_spacing(scale);
 
        let mut total_width = 0.0_f32;
        for word in text.split(config.word_seperator) {
            let width = measure_word_width::<TFont>(word,scale,letter_spacing);
            total_width += width + word_spacing;
        }
        total_width -= word_spacing;

        let mut x = (config.position.0 - total_width).round();
        let y = config.position.1.round();

        let color = &config.color;
        for word in text.split(config.word_seperator) {
            for character in word.chars() {
                x += self.draw_glyph::<TFont>(character,x,y,scale,color) + letter_spacing;
            }
            x += word_spacing - letter_spacing;
        }
    }

    fn draw_text_centered<TFont: FontDefinition>(&mut self,text: &str,config: TextRenderConfig) {
        let scale = validate_scale(config.scale);
        let word_spacing = TFont::get_word_spacing(scale);
        let letter_spacing = TFont::get_letter_spacing(scale);

        let mut total_width = 0.0_f32;
        for word in text.split(config.word_seperator) {
            let width = measure_word_width::<TFont>(word,scale,letter_spacing);
            total_width += width + word_spacing;
        }

        let mut x = config.position.0 - ((total_width - word_spacing) * 0.5).round();
        let y = config.position.1 - (TFont::get_line_height(scale * config.line_height) * 0.5).round();

        let color = &config.color;
        for word in text.split(config.word_seperator) {
            for character in word.chars() {
                x += self.draw_glyph::<TFont>(character,x,y,scale,color) + letter_spacing;
            }
            x += word_spacing - letter_spacing;
        }
    }

    pub fn get_instance_buffer_len(&self) -> usize {
        return self.context.get_text_pipeline().instance_buffer.len();
    }

    pub fn draw_text<TFont: FontDefinition>(&mut self,text: &str,config: TextRenderConfig) {
        if !self.validate_texture::<TFont>() {
            return;
        }

        let range_start = self.get_instance_buffer_len();

        match config.behavior {
            TextRenderBehavior::LTR => {
                self.draw_text_ltr::<TFont>(text,config)
            },
            TextRenderBehavior::Centered => {
                self.draw_text_centered::<TFont>(text,config)
            },
            TextRenderBehavior::RTL => {
                self.draw_text_rtl::<TFont>(text,config)
            },
            TextRenderBehavior::LineBreakingLTR { line_width: max_width } => {
                self.draw_text_line_breaking_ltr::<TFont>(text,config,max_width)
            }
        }

        let range_end = self.get_instance_buffer_len();

        if range_start == range_end {
            return;
        }

        self.render_pass.draw_indexed(0..TextPipeline::INDEX_BUFFER_SIZE,0,Range {
            start: range_start as u32,
            end: range_end as u32
        });
    }
}

#[repr(C)]
#[derive(Copy,Clone,Debug,Default,Pod,Zeroable)]
pub struct GlyphVertex {
    pub position: [f32;2],
}

#[repr(C)]
#[derive(Copy,Clone,Debug,Default,Pod,Zeroable)]
pub struct GlyphInstance {
    pub position: [f32;2],
    pub size: [f32;2],
    pub uv_position: [f32;2],
    pub uv_size: [f32;2],
    pub color: [u8;4],
}

#[non_exhaustive]
struct ATTR;

impl ATTR {
    pub const VERTEX_POSITION: u32 = 0;
    pub const INSTANCE_POSITION: u32 = 1;
    pub const SIZE: u32 = 2;
    pub const UV_POS: u32 = 3;
    pub const UV_SIZE: u32 = 4;
    pub const COLOR: u32 = 5;
}

impl GlyphVertex {
    const ATTRS: [wgpu::VertexAttribute;1] = wgpu::vertex_attr_array![
        ATTR::VERTEX_POSITION => Float32x2,
    ];

    pub fn get_buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        return wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRS,
        }
    }
}

impl GlyphInstance {
    const ATTRS: [wgpu::VertexAttribute;5] = wgpu::vertex_attr_array![
        ATTR::INSTANCE_POSITION => Float32x2,
        ATTR::SIZE => Float32x2,
        ATTR::UV_POS => Float32x2,
        ATTR::UV_SIZE => Float32x2,
        ATTR::COLOR => Unorm8x4,
    ];

    pub fn get_buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        return wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRS,
        }
    }
}
