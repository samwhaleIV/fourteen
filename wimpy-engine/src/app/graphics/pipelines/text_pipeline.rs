use wgpu::*;
use wgpu::util::{BufferInitDescriptor,DeviceExt};
use std::marker::PhantomData;
use std::ops::Range;
use bytemuck::{Pod,Zeroable};
use crate::{WimpyColor,WimpyRect,WimpyVec};
use crate::app::graphics::{*,constants::*};
use super::core::*;

const VERTEX_BUFFER_INDEX: u32 = 0;
const INSTANCE_BUFFER_INDEX: u32 = 1;
const INDEX_BUFFER_SIZE: u32 = 6;
const TEXTURE_BIND_GROUP_INDEX: u32 = 0;
const UNIFORM_BIND_GROUP_INDEX: u32 = 1;

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

    pub fn create<TConfig>(
        graphics_provider: &GraphicsProvider,
        texture_layout: &BindGroupLayout,
        uniform_layout: &BindGroupLayout,
    ) -> Self
    where
        TConfig: GraphicsContextConfig
    {
        let device = graphics_provider.get_device();

        let shader = &device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Text Pipeline Shader"),
            source: ShaderSource::Wgsl(include_str!("shaders/text_pipeline.wgsl").into())
        });

        let render_pipeline_layout = &device.create_pipeline_layout(&PipelineLayoutDescriptor {
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
            primitive_state: &PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
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

        let indices: [u32;INDEX_BUFFER_SIZE as usize] = [
            0,1,2,
            2,1,3
        ];

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor{
            label: Some("Text Pipeline Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::INDEX
        });

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor{
            label: Some("Text Pipeline Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX
        });

        let instance_buffer = DoubleBuffer::new(
            device.create_buffer(&BufferDescriptor{
                label: Some("Text Pipeline Instance Buffer"),
                size: TConfig::TEXT_PIPELINE_BUFFER_SIZE as BufferAddress,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        );

        Self {
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
    uv_scalar: WimpyVec,
}

pub struct TextLine<'a> {
    pub text: &'a str,
    pub color: WimpyColor
}

#[derive(Clone,Copy)]
pub enum TextDirection {
    LeftToRight,
    RightToLeft
}

impl TextDirection {
    fn is_ltr(self) -> bool {
        match self {
            TextDirection::LeftToRight => true,
            TextDirection::RightToLeft => false,
        }
    }
}

pub struct TextRenderConfig {
    pub position: WimpyVec,
    pub color: WimpyColor,
    pub scale: f32,
    pub line_height_scale: f32,
    pub word_seperator: char,
}

fn validate_scale(scale: f32) -> f32 {
    scale.round().max(1.0)
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
            IndexFormat::Uint32
        );

        render_pass.set_vertex_buffer(
            VERTEX_BUFFER_INDEX,
            text_pipeline.vertex_buffer.slice(..)
        );

        render_pass.set_vertex_buffer(
            INSTANCE_BUFFER_INDEX,
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

        Self {
            uv_scalar: WimpyVec::ONE,
            tex,
            context,
            render_pass,
        }
    }
}

struct TextRenderer<'a,TFont> {
    scale: f32,
    word_spacing: f32,
    letter_spacing: f32,
    line_height: f32,
    pos: WimpyVec,
    color: WimpyColor,
    word_seperator: char,
    uv_scalar: WimpyVec,
    buffer: &'a mut DoubleBuffer<GlyphInstance>,
    _phantom: PhantomData<TFont>
}

impl<'a,TFont> TextRenderer<'a,TFont>
where
    TFont: FontDefinition
{
    fn new(
        config: TextRenderConfig,
        uv_scalar: WimpyVec,
        buffer: &'a mut DoubleBuffer<GlyphInstance>,
    ) -> Self {
        let scale = validate_scale(config.scale);
        Self {
            scale,
            word_spacing: TFont::get_word_spacing(scale),
            letter_spacing: TFont::get_letter_spacing(scale),
            line_height: TFont::get_line_height(scale * config.line_height_scale),
            pos: config.position,
            color: config.color,
            word_seperator: config.word_seperator,
            uv_scalar,
            buffer,
            _phantom: PhantomData,
        }
    }

    fn measure_word_width(&self,word: &str) -> f32 {
        let mut width = 0.0_f32;
        for character in word.chars() {
            let glyph = TFont::get_glyph(character);
            width += glyph.width as f32 * self.scale + self.letter_spacing;
        }
        width - self.letter_spacing
    }

    fn measure_text_width(&self,text: &str) -> f32 {
        let mut total_width = 0.0_f32;
        for word in text.split(self.word_seperator) {
            let width = self.measure_word_width(word);
            total_width += width + self.word_spacing;
        }
        total_width - self.word_spacing
    }

    fn draw_text(&mut self,text: &str,row: usize,ltr: bool) {

        let mut pos = WimpyVec {
            x: match ltr {
                true => self.pos.x,
                false => self.pos.x - self.measure_text_width(text),
            }.round(),
            y: self.line_height.mul_add(row as f32,self.pos.y).round(),
        };

        for word in text.split(self.word_seperator) {
            for char in word.chars() {
                pos.x += self.draw_glyph(char,pos) + self.letter_spacing;
            }
            pos.x += self.word_spacing - self.letter_spacing;
        }
    }

    fn draw_text_line_breaking_ltr(&mut self,text: &str,max_width: f32) {
        let mut pos = self.pos.round();

        let x_start = pos.x;
        let max_x = x_start + max_width;

        for word in text.split(self.word_seperator) {
            if pos.x + self.measure_word_width(word) > max_x {
                pos.x = x_start;
                pos.y += self.line_height;
            }
            for character in word.chars() {
                pos.x += self.draw_glyph(character,pos) + self.letter_spacing;
            }
            pos.x += self.word_spacing - self.letter_spacing;
        }
    }

    fn draw_text_centered(&mut self,text: &str,) {
        let total_width = self.measure_text_width(text);

        let mut pos = WimpyVec {
            x: (total_width - self.word_spacing).mul_add(-0.5,self.pos.x).round(),
            y: self.line_height.mul_add(-0.5,self.pos.y).round(),
        };

        for word in text.split(self.word_seperator) {
            for character in word.chars() {
                pos.x += self.draw_glyph(character,pos) + self.letter_spacing;
            }
            pos.x += self.word_spacing - self.letter_spacing;
        }
    }

    fn draw_glyph(&mut self,char: char,pos: WimpyVec) -> f32 {
        let glyph = TFont::get_glyph(char);
        if glyph.width == 0 {
            return 0.0;
        }

        let width = glyph.width as f32;
        let height = glyph.height as f32;

        let src = WimpyRect::from([
            glyph.x as f32,
            glyph.y as f32,
            width,
            height
        ]) * self.uv_scalar;

        let dst = WimpyRect {
            position: WimpyVec {
                x: pos.x,
                y: (glyph.y_offset as f32).mul_add(self.scale,pos.y)
            },
            size: WimpyVec::from([width,height]) * self.scale
        }.origin_top_left_to_center();

        let glyph_instance = GlyphInstance {
            position: dst.position.into(),
            size: dst.size.into(),
            uv_position: src.position.into(),
            uv_size: src.size.into(),
            color: self.color.into(),
        };

        self.buffer.push(glyph_instance);

        dst.width()
    }
}

impl PipelineTextPass<'_,'_> {
    fn validate_texture<TFont: FontDefinition>(&mut self) -> bool {
        let current_texture = self.tex;
        let target_texture = TFont::get_texture(self.context.textures);
        let target_texture_ref = target_texture.get_ref();

        if current_texture.get_ref() != target_texture.get_ref() {
            match self.context.frame_cache.get(target_texture_ref) {
                Ok(texture_container) => self.context.set_texture_bind_group(
                    TEXTURE_BIND_GROUP_INDEX,
                    &mut self.render_pass,
                    &BindGroupCacheIdentity::SingleChannel {
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
            self.uv_scalar = WimpyVec::ONE / WimpyVec::from(self.tex.size()) * scale;
        }

        return true;
    }

    fn draw_text_internal<TFont,F>(
        &mut self,
        config: TextRenderConfig,
        f_renderer: F
    )
    where
        TFont: FontDefinition,
        F: FnOnce(&mut TextRenderer<'_,TFont>)
    {
        if !self.validate_texture::<TFont>() {
            return;
        }

        let pipeline = &mut self.context.get_text_pipeline_mut();
        let range_start = pipeline.instance_buffer.len();

        let mut renderer = TextRenderer::<TFont>::new(config,self.uv_scalar,&mut pipeline.instance_buffer);
        f_renderer(&mut renderer);

        let range_end = self.context.get_text_pipeline().instance_buffer.len();
        if range_start == range_end {
            return;
        }
        self.render_pass.draw_indexed(0..INDEX_BUFFER_SIZE,0,Range {
            start: range_start as u32,
            end: range_end as u32
        });
    }

    pub fn draw_text<TFont: FontDefinition>(&mut self,lines: &[&str],direction: TextDirection,config: TextRenderConfig) {
        self.draw_text_internal::<TFont,_>(config,|r|{
            let ltr = direction.is_ltr();
            for (i,&line) in lines.iter().enumerate() {
                r.draw_text(line,i,ltr);
            }
        });
    }

    pub fn draw_text_wrapping<TFont: FontDefinition>(&mut self,text: &str,max_width: f32,config: TextRenderConfig) {
        self.draw_text_internal::<TFont,_>(config,|r|{
            r.draw_text_line_breaking_ltr(text,max_width)
        });
    }

    pub fn draw_text_centered<TFont: FontDefinition>(&mut self,text: &str,config: TextRenderConfig) {
        self.draw_text_internal::<TFont,_>(config,|r|{
            r.draw_text_centered(text);
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
    const ATTRS: [VertexAttribute;1] = vertex_attr_array![
        ATTR::VERTEX_POSITION => Float32x2,
    ];

    pub fn get_buffer_layout<'a>() -> VertexBufferLayout<'a> {
        return VertexBufferLayout {
            array_stride: size_of::<Self>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRS,
        }
    }
}

impl GlyphInstance {
    const ATTRS: [VertexAttribute;5] = vertex_attr_array![
        ATTR::INSTANCE_POSITION => Float32x2,
        ATTR::SIZE => Float32x2,
        ATTR::UV_POS => Float32x2,
        ATTR::UV_SIZE => Float32x2,
        ATTR::COLOR => Unorm8x4,
    ];

    pub fn get_buffer_layout<'a>() -> VertexBufferLayout<'a> {
        return VertexBufferLayout {
            array_stride: size_of::<Self>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &Self::ATTRS,
        }
    }
}
