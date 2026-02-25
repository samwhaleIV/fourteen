use std::ops::Mul;

use crate::{app::{graphics::*, input::{Impulse, ImpulseEvent, ImpulseState, MousePressState}, *},*};

pub struct PlaceholderApp {
    test_texture: TextureFrame,
    line_start: Option<WimpyVec>,
    lines: Vec<[LinePoint;2]>,
    offset: WimpyVec,
    in_movement_mode: bool
}

impl PlaceholderApp {
    fn pressed_enter(&mut self,context: &mut WimpyContext) -> bool {
        let mut toggle = false;
        for event in context.input.iter_recent_events() {
            match event {
                ImpulseEvent {
                    impulse: Impulse::Confirm,
                    state: ImpulseState::Pressed,
                } => {
                    toggle = true;
                    break;
                },
                _ => {}
            }
        }
        return toggle;
    }
}

fn map_mouse_delta_for_graph(value: f32) -> i8 {
    const MAX_DELTA: f32 = 25.0;
    const SCALE: f32 = 127.0 / MAX_DELTA;
    (value.clamp(-MAX_DELTA,MAX_DELTA) * SCALE).round() as i8
}

impl<IO> WimpyApp<IO> for PlaceholderApp
where
    IO: WimpyIO
{
    async fn load(context: &mut WimpyContext) -> Self {

        let render_config = context.debug.get_render_config();
        render_config.top_left = Pane {
            size: WimpyVec::new(400.0,400.0),
            layout: PaneLayout::single(SubPane {
                item: PaneItem::Graph {
                    width: GraphWidth::Half,
                    layers: GraphLayers::Dual { layers: [
                        GraphLayer {
                            id: GraphID::One,
                            color: WimpyNamedColor::Red
                        },
                        GraphLayer {
                            id: GraphID::Two,
                            color: WimpyNamedColor::Blue
                        },
                    ] }
                },
                background_color: WimpyNamedColor::Gray,
                background_opacity: WimpyOpacity::Percent90,
            })
        };

        return Self {
            in_movement_mode: false,
            lines: Vec::with_capacity(64),
            test_texture: context.load_image_or_default::<IO>("wimpy/color-test").await,
            line_start: None,
            offset: WimpyVec::ZERO
        };
    }

    fn update(&mut self,context: &mut WimpyContext) {

        let pressed_enter = self.pressed_enter(context);

        let mouse = context.input.get_virtual_mouse_mut();
        
        // Start render ...
        if pressed_enter {
            self.in_movement_mode = !self.in_movement_mode;
            if self.in_movement_mode {
                mouse.queue_camera_mode();
            } else {
                mouse.queue_interaction_mode();
            }
        }

        context.debug.set_label_text_fmt(
            LabelID::One,
            format_args!("x: {:.0} y: {:.0} pressed: {:?}",mouse.position().x,mouse.position().y,mouse.left_is_pressed())
        );

        context.debug.set_label_text_fmt(
            LabelID::Two,
            format_args!("dx: {:.0} dy: {:.0}",mouse.delta().x,mouse.delta().y)
        );

        context.debug.set_graph_value(GraphID::One,map_mouse_delta_for_graph(mouse.delta().x));
        context.debug.set_graph_value(GraphID::Two,map_mouse_delta_for_graph(mouse.delta().y));

        match mouse.get_active_mode() {
            input::MouseMode::Interface => {
                match mouse.left_press_state() {
                    MousePressState::JustPressed | MousePressState::Pressed => {
                        if self.line_start.is_none() {
                            self.line_start = Some(mouse.position())
                        }
                    },
                    MousePressState::JustReleased | MousePressState::Released => {
                        if let Some(start) = self.line_start.take() {
                            let end = mouse.position();
                            self.lines.push([
                                LinePoint {
                                    point: start,
                                    color: WimpyColor::RED,
                                },
                                LinePoint {
                                    point: end,
                                    color: WimpyColor::GREEN,
                                }
                            ])
                        }
                    },
                }
            },
            input::MouseMode::Camera => {
                self.offset += mouse.delta();
            },
        };

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
                    size_offset: Size::from(0),
                },
                y: LayoutDimension {
                    position: Position::center_of_parent(),
                    size: texture.height().into(),
                    size_offset: Size::from(0),
                },
            };

            pipeline_pass_2d.set_sampler_mode(SamplerMode::NearestClamp);

            let mut destination = layout.compute(output.frame.area());
            destination.position += self.offset;

            pipeline_pass_2d.draw(&texture,&[DrawData2D {
                destination,
                source: WimpyRect::ONE,
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
