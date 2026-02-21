use crate::{shared::*,app::{*,graphics::{*,fonts::*},}};
use std::fmt::Write;

pub struct PlaceholderApp {
    test_texture: TextureFrame,
}

impl<IO> WimpyApp<IO> for PlaceholderApp
where
    IO: WimpyIO
{
    async fn load(context: &mut WimpyContext) -> Self {

        context.debug.set_render_config_top_left(PaneLayout::One {
            items: [PaneItem::Label { channel: LabelChannel::One, color: WimpyColor::WHITE }]
        });

        return Self {
            test_texture: context.load_image_or_default::<IO>("test-namespace/test").await
        };
    }

    fn update(&mut self,context: &mut WimpyContext) {

        // Start render ...

        let mouse = context.input.get_virtual_mouse();
        context.debug.set_label_text_fmt(
            LabelChannel::One,
            format_args!("x: {:.0} y: {:.0}",mouse.position().x,mouse.position().y)
        );

        let mut output = match context.graphics.create_output_builder(WimpyColor::BLACK) {
            Ok(value) => value,
            Err(surface_error) => {
                log::error!("Could not create output surface: {:?}",surface_error);
                return;
            },
        };

        'output_pass: {
            let Ok(mut render_pass) = output.builder.create_render_pass(&output.frame) else {
                break 'output_pass;
            };
            let mut pipeline_pass_2d = render_pass.set_pipeline_2d();

            let texture = self.test_texture;

            let layout = WimpyLayout {
                x: LayoutDimension {
                    position: Position::center_of_parent(),
                    size: texture.width().into(),
                    size_offset: 0.into(),
                },
                y: LayoutDimension {
                    position: Position::center_of_parent(),
                    size: texture.height().into(),
                    size_offset: 0.into(),
                },
            };

            pipeline_pass_2d.set_sampler_mode(SamplerMode::NearestClamp);

            let destination = layout.compute(output.frame.area());

            pipeline_pass_2d.draw(&texture,&[DrawData2D {
                destination,
                source: WimpyArea::ONE,
                color: WimpyColor::WHITE,
                rotation: 0.0,
            }]);

            context.debug.render(&mut render_pass);
        }

        output.present_output_surface();
    }
}

pub struct PlaceholderConfig;

const fn mb_to_b(value: usize) -> usize {
    value * 1000000
}

impl GraphicsContextConfig for PlaceholderConfig {
    // If a vertex is 32 bytes, there is 31,250 vertices per megabyte.

    const MODEL_CACHE_VERTEX_BUFFER_SIZE: usize = mb_to_b(1);
    const MODEL_CACHE_INDEX_BUFFER_SIZE: usize = mb_to_b(1);

    const UNIFORM_BUFFER_SIZE: usize = 16384;

    const INSTANCE_BUFFER_SIZE_2D: usize = mb_to_b(1);
    const INSTANCE_BUFFER_SIZE_3D: usize = mb_to_b(1);
    
    const TEXT_PIPELINE_BUFFER_SIZE: usize = mb_to_b(1);
}
