use crate::app::*;
use crate::app::graphics::*;
use crate::shared::*;

pub struct PlaceholderApp {
    test_texture: TextureFrame
}

impl<IO> WimpyApp<IO> for PlaceholderApp
where
    IO: WimpyIO
{
    async fn load(context: &mut WimpyContext<'_>) -> Self {
        return Self {
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

        'first_pass: {
            let Ok(mut render_pass) = output.builder.start_pass_2d(&output.frame) else {
                break 'first_pass;
            };

            let texture = self.test_texture;

            let size: Size = 400.into();

            let layout = WimpyLayout {
                x: LayoutDimension {
                    position: Position::center_of_parent(),
                    size,
                    size_offset: 0.into(),
                },
                y: LayoutDimension {
                    position: Position::center_of_parent(),
                    size,
                    size_offset: 0.into(),
                },
            };

            render_pass.set_sampler_mode(SamplerMode::NearestClamp);

            render_pass.draw(&texture,&[DrawData2D {
                destination: layout.compute(output.frame.area()),
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

    const MODEL_CACHE_VERTEX_BUFFER_SIZE: usize = mb_to_b(10);
    const MODEL_CACHE_INDEX_BUFFER_SIZE: usize = mb_to_b(10);

    const UNIFORM_BUFFER_SIZE: usize = 65536;

    const INSTANCE_BUFFER_SIZE_2D: usize = mb_to_b(5);
    const INSTANCE_BUFFER_SIZE_3D: usize = mb_to_b(5);
}
