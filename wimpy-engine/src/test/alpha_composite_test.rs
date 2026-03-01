use std::iter;
use crate::{app::{graphics::*, *},*};

/// Test sRGB texture loading, presentation, internal color (named or 'WimpyColorSrgb') sRGB translation, and linear alpha compositing behavior
/// 
/// Shader expects linear texture data. OK to store in sRGB formats or linear, wgpu will convert
/// 
/// Linear alpha compositing is desired rather than the all-too-common but incorrect post-sRGB/gamma-on-gamma blend
pub struct SrgbTest {
    srgb_test_texture: TextureFrame
}

const STRIP_COUNT: usize = 8;
const TOTAL_AREA: f32 = 512.0;
const STRIP_STRIDE: f32 = TOTAL_AREA / STRIP_COUNT as f32;
const STRIP_WIDTH: f32 = STRIP_STRIDE * 0.5;
const STRIP_WIDTH_OFFSET: f32 = STRIP_STRIDE * -0.25;

const H: u8 = u8::MAX;
const L: u8 = u8::MIN;
const M: u8 = u8::MAX / 2;

const STRIPS: &[WimpyColorSrgb;STRIP_COUNT] = &[
    WimpyColorSrgb { r: H, g: L, b: L, a: H },  // Red
    WimpyColorSrgb { r: L, g: H, b: L, a: H },  // Green
    WimpyColorSrgb { r: L, g: L, b: H, a: H },  // Blue
    WimpyColorSrgb { r: L, g: H, b: H, a: H },  // Cyan,
    WimpyColorSrgb { r: H, g: L, b: H, a: H },  // Magenta
    WimpyColorSrgb { r: H, g: H, b: L, a: H },  // Yellow
    WimpyColorSrgb { r: L, g: L, b: L, a: H },  // Black
    WimpyColorSrgb { r: M, g: M, b: M, a: H },  // Middle gray
];

impl<IO> WimpyApp<IO> for SrgbTest
where
    IO: WimpyIO
{
    async fn load(context: &mut WimpyContext) -> Self {
        Self {
            srgb_test_texture: context.get_image::<IO>("wimpy/srgb-test").await,
        }
    }

    fn update(&mut self,context: &mut WimpyContext) {
        let Some(mut output) = context.graphics.create_output_builder(WimpyNamedColor::Black) else {
            return;
        };

        'output_pass: {
            let Ok(mut render_pass) = output.builder.create_render_pass(&output.frame) else {
                break 'output_pass;
            };
            let mut pipeline_pass_2d = render_pass.set_pipeline_2d();

            let layout = WimpyLayout {
                x: LayoutDimension {
                    position: Position::center_of_parent(),
                    size: TOTAL_AREA.into(),
                    size_offset: Size::from(0),
                },
                y: LayoutDimension {
                    position: Position::center_of_parent(),
                    size: TOTAL_AREA.into(),
                    size_offset: Size::from(0),
                },
            };

            let destination = layout.compute(output.frame.area());

            pipeline_pass_2d.draw(&self.srgb_test_texture,iter::once(DrawData2D {
                destination: output.frame.area(),
                source: WimpyRect::ONE,
                color: WimpyColorLinear::WHITE,
                rotation: 0.0,
            }));

            pipeline_pass_2d.draw_untextured(iter::once(
                DrawData2D {
                    destination,
                    source: WimpyRect::ONE,
                    color: WimpyColorLinear::WHITE,
                    rotation: 0.0,
                }
            ).chain(
                (0..STRIP_COUNT).map(|i|{
                        let destination = WimpyRect {
                        position: destination.position + WimpyVec {
                            x: (i as f32 + 0.5).mul_add(STRIP_STRIDE,STRIP_WIDTH_OFFSET),
                            y: 0.0,
                        },
                        size: WimpyVec {
                            x: STRIP_WIDTH,
                            y: destination.size.y
                        }
                    };
                    DrawData2D {
                        destination,
                        source: WimpyRect::ONE,
                        color: STRIPS[i].into_linear(),
                        rotation: 0.0
                    }
                })
            ).chain(
                (0..STRIP_COUNT).map(|i|{
                    let destination = WimpyRect {
                        position: destination.position + WimpyVec {
                            x: 0.0,
                            y: (i as f32 + 0.5).mul_add(STRIP_STRIDE,STRIP_WIDTH_OFFSET),
                        },
                        size: WimpyVec {
                            x: destination.size.x,
                            y: STRIP_WIDTH
                        }
                    };
                    let mut color: WimpyColorLinear = STRIPS[i].into_linear();
                    color.a = 0.5;
                    DrawData2D {
                        destination,
                        source: WimpyRect::ONE,
                        color,
                        rotation: 0.0
                    }
                })
            ));
        }

        output.present_output_surface();
    }
}
