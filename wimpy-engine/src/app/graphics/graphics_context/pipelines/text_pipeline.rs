mod creation;
pub mod engine_fonts;

mod shader_definitions;
pub use shader_definitions::*;
use super::*;

pub struct TextPipeline {
    pipelines: PipelineVariants,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    instance_buffer: DoubleBuffer<GlyphInstance>,
}

pub struct PipelineTextPass<'a,'frame> {
    context: &'a mut RenderPassContext<'frame>,
    render_pass: &'a mut RenderPass<'frame>,
    current_texture: TextureFrame,
    current_texture_uv_size: (f32,f32)
}

pub enum TextBehavior {
    LTR,
    RTL,
    Centered,
    LineBreakingLTR {
        max_width: u32
    }
}

pub struct TextRenderConfig {
    position: (f32,f32),
    scale: f32,
    color: WimpyColor,
    line_height: f32,
    word_seperator: char,
    behavior: TextBehavior
}

#[derive(Default)]
pub struct GlyphArea {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub y_offset: i16
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

    fn get_line_height(scale: f32,line_height: f32) -> f32 {
        (Self::LINE_HEIGHT * line_height * scale).round().max(Self::LINE_HEIGHT + 1.0)
    }
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

        let current_texture = context.textures.transparent_black.clone();

        return Self {
            current_texture_uv_size: current_texture.get_output_uv_size(),
            context,
            render_pass,
            current_texture: current_texture
        }
    }
}

impl PipelineTextPass<'_,'_> {
    fn validate_texture<TFont: FontDefinition>(&mut self) -> bool {
        let current_texture = self.current_texture;
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
            self.current_texture = target_texture;
            self.current_texture_uv_size = self.current_texture.get_output_uv_size();
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
            x: glyph.x as f32,
            y: glyph.y as f32,
            width,
            height,
        }.multiply_2d(self.current_texture_uv_size);

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

    fn draw_text_line_breaking_ltr<TFont: FontDefinition>(&mut self,text: &str,config: TextRenderConfig,max_width: u32) {
        let scale = validate_scale(config.scale);
        todo!();
    }

    fn draw_text_ltr<TFont: FontDefinition>(&mut self,text: &str,config: TextRenderConfig) {
        let scale = validate_scale(config.scale);
        todo!();
    }

    fn draw_text_rtl<TFont: FontDefinition>(&mut self,text: &str,config: TextRenderConfig) {
        let scale = validate_scale(config.scale);
        todo!();
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
        let y = config.position.1 - (TFont::get_line_height(scale,config.line_height) * 0.5).round();

        let color = &config.color;
        for word in text.split(config.word_seperator) {
            for character in word.chars() {
                x += self.draw_glyph::<TFont>(character,x,y,scale,color);
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
            TextBehavior::LTR | TextBehavior::LineBreakingLTR {
                max_width: 0
            } => {
                self.draw_text_ltr::<TFont>(text,config)
            },
            TextBehavior::Centered => {
                self.draw_text_centered::<TFont>(text,config)
            },
            TextBehavior::RTL => {
                self.draw_text_rtl::<TFont>(text,config)
            },
            TextBehavior::LineBreakingLTR { max_width } => {
                self.draw_text_line_breaking_ltr::<TFont>(text,config,max_width)
            }
        }

        let range_end = self.get_instance_buffer_len();

        if range_start != range_end {
            return;
        }

        self.render_pass.draw_indexed(0..TextPipeline::INDEX_BUFFER_SIZE,0,Range {
            start: range_start as u32,
            end: range_end as u32
        });
    }
}
