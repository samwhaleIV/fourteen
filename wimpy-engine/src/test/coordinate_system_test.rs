use glam::Vec3;

use crate::{app::{graphics::*, input::Impulse, *}, world::{CameraPositionStrategy, CameraPositionUpdate, WimpyCamera}, *};

/// Test sRGB texture loading, presentation, internal color (named or 'WimpyColorSrgb') sRGB translation, and linear alpha compositing behavior
/// 
/// Shader expects linear texture data. OK to store in sRGB formats or linear, wgpu will convert
/// 
/// Linear alpha compositing is desired rather than the all-too-common but incorrect post-sRGB/gamma-on-gamma blend
pub struct CoordinateSystemTest {
    camera: WimpyCamera,
    lines: Vec<LinePoint3D>
}

const LINE_COUNT: usize = 10;

const BACKGROUND_COLOR: WimpyColorLinear =  WimpyColorLinear::from_srgb(64,     64,     64,     255);
const LINE_COLOR: WimpyColorLinear =        WimpyColorLinear::from_srgb(80,     80,     80,     255);
const X_AXIS_COLOR: WimpyColorLinear =      WimpyColorLinear::from_srgb(180,    0,      0,      255);
const Y_AXIS_COLOR: WimpyColorLinear =      WimpyColorLinear::from_srgb(80,     180,    0,      255);
const Z_AXIS_COLOR: WimpyColorLinear =      WimpyColorLinear::from_srgb(0,      0,      180,    255);

fn generate_lines() -> Vec<LinePoint3D> {

    let mut line_container = Vec::with_capacity(LINE_COUNT * 2 * 2 + 1);
    const OFFSET: f32 = 0.0; //todo.. for centering

    const SIZE: f32 = LINE_COUNT as f32;
    const HALF_SIZE: f32 = SIZE * 0.5;

    // Z Line
    line_container.push(LinePoint3D {
        point: Vec3::new(0.0,0.0,-HALF_SIZE),
        color: Z_AXIS_COLOR,
    });
    line_container.push(LinePoint3D {
        point: Vec3::new(0.0,0.0,HALF_SIZE),
        color: Z_AXIS_COLOR,
    });

    for i in 0..LINE_COUNT {
        let is_middle_line: bool = false;
        let (x_color,y_color) = match is_middle_line {
            true => {
                (X_AXIS_COLOR,Y_AXIS_COLOR)
            },
            false => {
                (LINE_COLOR,LINE_COLOR)
            },
        };

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

    line_container
}

impl<IO> WimpyApp<IO> for CoordinateSystemTest
where
    IO: WimpyIO
{
    async fn load(context: &mut WimpyContext) -> Self {
        context.input.get_virtual_mouse_mut().queue_camera_mode();
        Self {
            camera: Default::default(),
            lines: generate_lines(),
            //TODO: Load coordinate test cube
            //coordinate_test_cube = context.assets.get_mo
        }
    }

    fn update(&mut self,context: &mut WimpyContext) {

        const MOVEMENT_UNITS_PER_SECOND: f32 = 2.0;

        let input = &context.input;
        let movement_delta = WimpyVec::from(input.get_axes()) * MOVEMENT_UNITS_PER_SECOND;

        use input::ImpulseState::*;
        let vertical_delta = match (
            input.get_state(Impulse::FocusLeft),
            input.get_state(Impulse::FocusRight)
        ) {
            (Pressed, Pressed) | (Released, Released) => 0.0,
            (Pressed, Released) => -1.0, // Fly down
            (Released, Pressed) => 1.0 //Fly up
        };

        const ANGLE_PER_PIXEL: f32 = 0.1;

        let mouse = context.input.get_virtual_mouse_mut();
        let look_delta = mouse.delta() * ANGLE_PER_PIXEL;

        self.camera.update_position(CameraPositionUpdate {
            position: CameraPositionStrategy::FreeCam {
                forward_movement: movement_delta.y * MOVEMENT_UNITS_PER_SECOND,
                side_movement: movement_delta.x * MOVEMENT_UNITS_PER_SECOND,
                vertical_movement: vertical_delta * MOVEMENT_UNITS_PER_SECOND,
            },
            delta_seconds: context.input.get_delta_seconds(),
            yaw_delta: look_delta.x,
            pitch_delta: look_delta.y,
        });

        let Some(mut output) = context.graphics.create_output_builder(WimpyNamedColor::Black) else {
            return;
        };

        'output_pass: {
            let Ok(mut render_pass) = output.builder.create_render_pass(&output.frame) else {
                break 'output_pass;
            };
            let camera_uniform = render_pass.create_camera_uniform(&self.camera,CameraPerspective::default());
            let mut lines_pass = render_pass.set_pipeline_lines_3d(camera_uniform);
            lines_pass.draw_list(&self.lines);
        }

        output.present_output_surface();
    }
}
