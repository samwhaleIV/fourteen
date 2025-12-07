use wgpu::TextureView;

use crate::{graphics::ViewProjection};

pub struct Camera {
    eye: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    aspect_ratio: f32,
    fov_y: f32,
    z_near: f32,
    z_far: f32
}

impl Camera {
    pub fn set_aspect_ratio(&mut self,render_target: &TextureView) {
        let texture = render_target.texture();
        let (width,height) = (texture.width(),texture.height());
        if width == 0 || height == 0 {
            log::warn!("Bad render target size, can't update camera aspect ratio. Size: {}x{}",width,height);
            return;
        }
        self.aspect_ratio = (width as f32) / (height as f32);
    }

    fn get_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fov_y),self.aspect_ratio,self.z_near,self.z_far);
        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }

    pub fn get_view_projection(&self) -> Box<ViewProjection> {
        let matrix = self.get_view_projection_matrix().into();
        let uniform = ViewProjection::create(matrix);
        return Box::new(uniform);
    }
}

impl Default for Camera {
    fn default() -> Self {
        return Self {
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect_ratio: 1.0,
            fov_y: 45.0,
            z_near: 0.1,
            z_far: 100.0
        };
    }
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);
