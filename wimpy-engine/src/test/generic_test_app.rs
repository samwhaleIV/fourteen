use crate::{app::{graphics::*, input::{Impulse, ImpulseEvent, ImpulseState, MousePressState}, *},*};

pub struct GenericTestApp {
    test_texture: TextureFrame,
    line_start: Option<WimpyVec>,
    lines: Vec<LinePoint2D>,
    offset: WimpyVec,
    in_movement_mode: bool
}

impl GenericTestApp {
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

const MAX_DELTA_INV: f32 = 1.0 / 25.0;

fn delta_norm(value: f32) -> f32 {
    (value * MAX_DELTA_INV).clamp(-1.0,1.0)
}

impl<IO> WimpyApp<IO> for GenericTestApp
where
    IO: WimpyIO
{
    async fn load(context: &mut WimpyContext) -> Self {

        let render_config = context.debug.get_render_config();
        render_config.top_right = Pane {
            size: WimpyVec::new(300.0,240.0),
            layout: PaneLayout::single(SubPane {
                item: PaneItem::Graph {
                    width: GraphWidth::Quarter,
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
                background_color: WimpyNamedColor::Black,
                background_opacity: WimpyOpacity::Percent95,
            })
        };

        return Self {
            in_movement_mode: false,
            lines: Vec::with_capacity(64),
            test_texture: context.load_image_or_default::<IO>("wimpy/blend-test").await,
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

        context.debug.set_graph_value(GraphID::One,delta_norm(mouse.delta().x));
        context.debug.set_graph_value(GraphID::Two,delta_norm(mouse.delta().y));

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
                            self.lines.push(LinePoint2D {
                                point: start,
                                color: WimpyColorLinear::RED,
                            });
                            self.lines.push(LinePoint2D {
                                point: end,
                                color: WimpyColorLinear::GREEN,
                            });
                        }
                    },
                }
            },
            input::MouseMode::Camera => {
                self.offset += mouse.delta();
            },
        };

        let Some(mut output) = context.graphics.create_output_builder(WimpyColorSrgb { r: 240, g: 240, b: 240, a: 255 }) else {
            return;
        };

        'output_pass: {
            let Ok(mut render_pass) = output.builder.create_render_pass(&output.frame) else {
                break 'output_pass;
            };
            let mut pipeline_pass_2d = render_pass.set_pipeline_2d();

            let texture = self.test_texture;

            let layout = WimpyLayout {
                x: LayoutDimension {
                    position: 5.into(),
                    size: texture.width().into(),
                    size_offset: Size::from(0),
                },
                y: LayoutDimension {
                    position: 5.into(),
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
                color: WimpyColorLinear::WHITE,
                rotation: 0.0,
            }]);

            // pipeline_pass_2d.draw_untextured(&[DrawData2D {
            //     destination,
            //     source: WimpyRect::ONE,
            //     color: WimpyColor::from((WimpyNamedColor::Red,WimpyOpacity::Percent50)),
            //     rotation: 0.0,
            // }]);

            context.debug.render(&mut render_pass);

            let mut lines_pass = render_pass.set_pipeline_lines_2d();
            lines_pass.draw_list(&self.lines);
        }

        output.present_output_surface();
    }
}
