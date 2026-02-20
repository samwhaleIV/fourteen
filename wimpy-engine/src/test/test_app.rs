use crate::app::*;
use crate::app::graphics::*;
use crate::shared::*;

pub struct PlaceholderApp {
    test_texture: TextureFrame,
    offset: (f32,f32)
}

impl<IO> WimpyApp<IO> for PlaceholderApp
where
    IO: WimpyIO
{
    async fn load(context: &mut WimpyContext) -> Self {
        return Self {
            offset: (0.0,0.0),
            test_texture: context.load_image_or_default::<IO>("test-namespace/test").await
        };
    }

    fn update(&mut self,context: &mut WimpyContext) {

        // Start render ...

        let mut output = match context.graphics.create_output_builder(WimpyColor::BLACK) {
            Ok(value) => value,
            Err(surface_error) => {
                log::error!("Could not create output surface: {:?}",surface_error);
                return;
            },
        };

        'output_pass: {
            let Ok(mut render_pass_builder) = output.builder.create_render_pass(&output.frame) else {
                break 'output_pass;
            };
            let mut render_pass_2d = render_pass_builder.set_pipeline_2d();

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

            render_pass_2d.set_sampler_mode(SamplerMode::NearestClamp);

            let mut destination = layout.compute(output.frame.area());
            destination.x += self.offset.0;
            destination.y += self.offset.1;

            render_pass_2d.draw(&texture,&[DrawData2D {
                destination,
                source: WimpyArea::ONE,
                color: WimpyColor::WHITE,
                rotation: 0.0,
            }]);
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
