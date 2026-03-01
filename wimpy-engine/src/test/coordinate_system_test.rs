use glam::{Mat4, Vec3};

use crate::{app::{graphics::{pipelines::pipeline_3d::TextureStrategy, *}, input::{Impulse, ImpulseEvent, ImpulseState, MouseMode}, wam::ModelData, *}, world::{CameraPositionStrategy, CameraPositionUpdate, WimpyCamera}, *};

pub struct CoordinateSystemTest {
    in_movement_mode: bool,
    camera: WimpyCamera,
    lines: Vec<LinePoint3D>,
    model: ModelData,
}

const LINE_COUNT: usize = 11;

const BACKGROUND_COLOR: WimpyColorLinear =  WimpyColorLinear::from_srgb(64,     64,     64,     255);
const LINE_COLOR: WimpyColorLinear =        WimpyColorLinear::from_srgb(80,     80,     80,     255);

const X_AXIS_COLOR: WimpyColorLinear =      WimpyColorLinear::from_srgb(180,    0,      0,      255);
const Y_AXIS_COLOR: WimpyColorLinear =      WimpyColorLinear::from_srgb(80,     180,    0,      255);
const Z_AXIS_COLOR: WimpyColorLinear =      WimpyColorLinear::from_srgb(0,      0,      180,    255);

fn generate_lines() -> Vec<LinePoint3D> {

    let mut line_container = Vec::with_capacity(LINE_COUNT * 2 * 2 + 1);

    const SIZE: f32 = LINE_COUNT as f32;
    const HALF_SIZE: f32 = SIZE * 0.5;

    const OFFSET: f32 = -HALF_SIZE + 0.5;

    for i in 0..LINE_COUNT+1 {
        let is_middle_line: bool = i == LINE_COUNT / 2;
        let (mut x_color,mut y_color) = match is_middle_line {
            true => {
                (X_AXIS_COLOR,Y_AXIS_COLOR)
            },
            false => {
                (LINE_COLOR,LINE_COLOR)
            },
        };

        if i == 0 {
            x_color = WimpyColorLinear::ORANGE;
            y_color = WimpyColorLinear::ORANGE;
        }

        let local_offset = i as f32;

        // X Line
        line_container.push(LinePoint3D {
            point: Vec3::new(OFFSET,OFFSET + local_offset,0.0),
            color: x_color,
        });
        line_container.push(LinePoint3D {
            point: Vec3::new(OFFSET + SIZE,OFFSET + local_offset,0.0),
            color: x_color,
        });

        // Y Line
        line_container.push(LinePoint3D {
            point: Vec3::new(OFFSET + local_offset,OFFSET,0.0),
            color: y_color,
        });
        line_container.push(LinePoint3D {
            point: Vec3::new(OFFSET + local_offset,OFFSET + SIZE,0.0),
            color: y_color,
        });
    }

    // Z Line
    line_container.push(LinePoint3D {
        point: Vec3::new(0.0,0.0,-HALF_SIZE),
        color: Z_AXIS_COLOR,
    });
    line_container.push(LinePoint3D {
        point: Vec3::new(0.0,0.0,HALF_SIZE),
        color: Z_AXIS_COLOR,
    });

    line_container
}

impl CoordinateSystemTest {
    fn pressed_enter(&self,context: &WimpyContext) -> bool {
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

impl<IO> WimpyApp<IO> for CoordinateSystemTest
where
    IO: WimpyIO
{
    async fn load(context: &mut WimpyContext) -> Self {
        let render_config = context.debug.get_render_config();
        render_config.top_left = Pane {
            size: WimpyVec::from(200),
            layout: PaneLayout::single(SubPane {
                item: PaneItem::Label {
                    channel: LabelID::One,
                    color: WimpyNamedColor::White,
                },
                background_color: WimpyNamedColor::Black,
                background_opacity: WimpyOpacity::Transparent,
            })
        };
        render_config.top_right = Pane {
            size: WimpyVec::from(200),
            layout: PaneLayout::single(SubPane {
                item: PaneItem::Label {
                    channel: LabelID::Two,
                    color: WimpyNamedColor::White,
                },
                background_color: WimpyNamedColor::Black,
                background_opacity: WimpyOpacity::Transparent,
            })
        };

        let model = context.get_model::<IO>("wimpy/models/coordinate-cube").await;

        Self {
            in_movement_mode: false,
            camera: Default::default(),
            lines: generate_lines(),
            model,
        }
    }

    fn update(&mut self,context: &mut WimpyContext) {
        const MOVEMENT_UNITS_PER_SECOND: f32 = 5.0;
        const ANGLE_PER_PIXEL: f32 = 0.15;

        if self.pressed_enter(context) {
            let mouse = context.input.get_virtual_mouse_mut();
            self.in_movement_mode = !self.in_movement_mode;
            if self.in_movement_mode {
                mouse.queue_camera_mode();
            } else {
                mouse.queue_interaction_mode();
            }
        }

        let input = &context.input;
        let movement_delta = WimpyVec::from(input.get_axes());

        use input::ImpulseState::*;
        let vertical_delta = match (
            input.get_state(Impulse::FocusLeft),
            input.get_state(Impulse::FocusRight)
        ) {
            (Pressed, Pressed) | (Released, Released) => 0.0,
            (Pressed, Released) => -1.0, // Fly down
            (Released, Pressed) => 1.0 //Fly up
        };

        let mouse = context.input.get_virtual_mouse_mut();
        let look_delta = mouse.delta() * ANGLE_PER_PIXEL;

        if context.input.get_virtual_mouse().get_active_mode() == MouseMode::Camera {
            self.camera.update_position(CameraPositionUpdate {
                position: CameraPositionStrategy::FreeCam {
                    forward_movement:   movement_delta.y * MOVEMENT_UNITS_PER_SECOND * -1.0,
                    side_movement:      movement_delta.x * MOVEMENT_UNITS_PER_SECOND,
                    vertical_movement:  vertical_delta *   MOVEMENT_UNITS_PER_SECOND,
                },
                delta_seconds: context.input.get_delta_seconds(),
                yaw_delta: look_delta.x,
                pitch_delta: -look_delta.y,
            });
        }

        let camera_position = self.camera.position();
        context.debug.set_label_fmt(LabelID::One,format_args!("x: {:.1}, y: {:.1}, z: {:.1}",
            camera_position.x,
            camera_position.y,
            camera_position.z
        ));

        context.debug.set_label_fmt(LabelID::Two,format_args!("xd: {:.1}, yd: {:.1}, zd: {:1}",
            movement_delta.x,
            movement_delta.y,
            vertical_delta
        ));

        let Some(mut output) = context.graphics.create_output_builder(BACKGROUND_COLOR) else {
            return;
        };

        'output_pass: {
            let Ok(mut render_pass) = output.builder.create_render_pass(&output.frame) else {
                break 'output_pass;
            };
            let camera_uniform = render_pass.create_camera_uniform(&self.camera,CameraPerspective::default());
            let mut lines_pass = render_pass.set_pipeline_lines_3d(camera_uniform);
            lines_pass.draw_list(&self.lines);

            let mut model_pass = render_pass.set_pipeline_3d(camera_uniform);
            model_pass.draw(&self.model,SamplerMode::NearestClamp,TextureStrategy::NoLightmap,std::iter::once(DrawData3D {
                transform: Mat4::IDENTITY,
                diffuse_color: WimpyColorLinear::WHITE,
                lightmap_color: WimpyColorLinear::WHITE,
            }));

            context.debug.render(&mut render_pass);
        }

        output.present_output_surface();
    }
}
