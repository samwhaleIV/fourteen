use crate::{app::{graphics::*, input::MousePressState, *}, shared::*};

pub struct PlaceholderApp {
    test_texture: TextureFrame,
    line_start: Option<(f32,f32)>,
    lines: Vec<[LinePoint;2]>
}

impl<IO> WimpyApp<IO> for PlaceholderApp
where
    IO: WimpyIO
{
    async fn load(context: &mut WimpyContext) -> Self {

        context.debug.set_render_config_top_left(PaneLayout::One {
            items: [PaneItem::Label { channel: LabelChannel::One }]
        });

        return Self {
            lines: Vec::with_capacity(64),
            test_texture: context.load_image_or_default::<IO>("test-namespace/test").await,
            line_start: None,
        };
    }

    fn update(&mut self,context: &mut WimpyContext) {

        // Start render ...

        let mouse = context.input.get_virtual_mouse();
        context.debug.set_label_text_fmt(
            LabelChannel::One,
            format_args!("x: {:.0} y: {:.0} pressed: {:?}",mouse.position().x,mouse.position().y,mouse.left_is_pressed())
        );

        match mouse.left_press_state() {
            MousePressState::JustPressed | MousePressState::Pressed => {
                if self.line_start.is_none() {
                    let start = mouse.position();
                    self.line_start = Some((start.x,start.y))
                }
            },
            MousePressState::JustReleased | MousePressState::Released => {
                if let Some(start) = self.line_start.take() {
                    let end = mouse.position();
                    self.lines.push([
                        LinePoint {
                            x: start.0,
                            y: start.1,
                            color: WimpyColor::RED,
                        },
                        LinePoint {
                            x: end.x,
                            y: end.y,
                            color: WimpyColor::GREEN,
                        }
                    ])
                }
            },
        }

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

            let mut lines_pass = render_pass.set_pipeline_lines();
            for line_set in &self.lines {
                lines_pass.draw(line_set);
            }
        }

        output.present_output_surface();
    }
}

pub struct PlaceholderConfig;

impl GraphicsContextConfig for PlaceholderConfig {
    // If a vertex is 32 bytes, there is 31,250 vertices per megabyte.
    const MODEL_CACHE_VERTEX_BUFFER_SIZE: usize = 16384;
    const MODEL_CACHE_INDEX_BUFFER_SIZE: usize = 16384;
    const UNIFORM_BUFFER_SIZE: usize = 16384;
    const INSTANCE_BUFFER_SIZE_2D: usize = 16384;
    const INSTANCE_BUFFER_SIZE_3D: usize = 16384;
    const TEXT_PIPELINE_BUFFER_SIZE: usize = 16384;
    const LINE_BUFFER_SIZE: usize = 16384;
}
