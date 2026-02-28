use glam::*;

/// Basic camera controller suitable for free cam or FPS controller
/// 
/// Uses the Blender right hand coordinate system, positive Z up
pub struct Camera {
    /// Eye point
    eye_position: Vec3,
    /// Yaw in radians (look left and right)
    yaw: f32,
    /// Pitch in radians (look up and down)
    pitch: f32,
    /// Viewport aspect ratio, as `w / h`
    aspect_ratio: f32,
    /// Vertical field of view in radians
    vertical_fov: f32,
    /// This is the coordinate space value for the near plane
    /// 
    /// The WGPU clip space is `0` (close) to `1` (far).
    clip_near: f32,
    /// This is the coordinate space value for the far plane
    /// 
    /// The WGPU clip space is `0` (close) to `1` (far).
    clip_far: f32,

    /// Current direction normal for yaw and pitch
    /// 
    /// In sync with `yaw` and `pitch`
    eye_angle: Vec3
}

pub enum CameraEyeData {
    FreeCam {
        /// Forward and back walking (Positive forward, negative back)
        /// 
        /// Look angle relative movement in units per second
        forward_movement: f32,

        /// Right and left walking (Positive right, negative left)
        /// 
        /// Look angle relative movement in units per second
        side_movement: f32,

        /// Up and down jumping/falling (Positive up, negative down)
        /// 
        /// Look angle relative movement in units per second
        vertical_movement: f32,
    },
    Manual {
        /// Manually positioned eye point
        /// 
        /// Useful if the camera is attached to a physics
        /// constrained object like a player controller
        eye: Vec3
    }
}

pub struct CameraUpdateData {
    /// Viewport aspect ratio, as `w / h`
    pub aspect_ratio: f32,

    pub eye: CameraEyeData,

    /// Vertical field of view, in degrees
    /// 
    /// `90` degrees is a reasonable starting point
    pub vertical_fov: f32,

    /// Time between displayed/executed frames in seconds
    pub delta_seconds: f32,

    /// Yaw look angle (left and right rotation)
    /// 
    /// Degrees per second
    pub yaw_delta: f32,

    /// Pitch look angle (up and down rotation)
    /// 
    /// Degrees per second
    pub pitch_delta: f32,
}

fn get_angle_normal(yaw: f32,pitch: f32) -> Vec3 {
    let (sin_yaw,cos_yaw) = yaw.sin_cos();
    let (sin_pitch,cos_pitch) = pitch.sin_cos();

    let angle_normal = Vec3::new(
        cos_pitch * sin_yaw,
        cos_pitch * cos_yaw,
        sin_pitch,
    ).normalize();
    angle_normal
}

pub fn reposition_eye(
    eye_position: Vec3,
    // Eye angle normal vector, originating on eye position
    eye_angle: Vec3,
    // Movement delta in units per second
    // [+x right, -x left], [+y forward, -y back], [+z up, -z down]
    delta: Vec3,
    // Delta seconds
    t: f32
) -> Vec3 {
    // Normal vector
    let forward = Vec3::new(
        eye_angle.x,
        eye_angle.y,
        0.0
    ).normalize();

    // Normal vector
    let side = Vec3::new(
        -forward.y,
        forward.x,
        0.0
    );

    let eye_delta = Vec3::new(
        (forward.x * delta.y) + (side.x * delta.x),
        (forward.y * delta.y) + (side.y * delta.x),
        delta.z
    );

    let eye = Vec3::new(
        eye_delta.x.mul_add(t,eye_position.x),
        eye_delta.y.mul_add(t,eye_position.y),
        eye_delta.z.mul_add(t,eye_position.z),
    );
    eye
}

impl Camera {

    pub fn update(&mut self,data: CameraUpdateData) {
        self.aspect_ratio = data.aspect_ratio;

        const RADS_PER_DEG: f32 = std::f32::consts::PI / 180.0;
        let rads_per_deg_per_s = RADS_PER_DEG * data.delta_seconds;

        self.yaw =  data.yaw_delta.mul_add(rads_per_deg_per_s,self.yaw);
        self.pitch = data.pitch_delta.mul_add(rads_per_deg_per_s,self.pitch);

        // TODO: Should angle of eye be updated before or after eye?
        self.eye_angle = get_angle_normal(self.yaw,self.pitch);

        self.eye_position = match data.eye {
            CameraEyeData::FreeCam {
                forward_movement,
                side_movement,
                vertical_movement
            } => {
                let movement_delta = Vec3::new(
                    side_movement,
                    forward_movement,
                    vertical_movement
                );
                reposition_eye(
                    self.eye_position,
                    self.eye_angle,
                    movement_delta,
                    data.delta_seconds
                )
            },
            CameraEyeData::Manual { eye } => eye,
        };

        self.vertical_fov = data.vertical_fov.to_radians();
    }
    pub fn get_matrix(&self) -> Mat4 {
        let look_projection = Mat4::look_to_rh(
            self.eye_position,
            self.eye_angle,
            Vec3::Z
        );

        let perspective_projection = Mat4::perspective_rh(
            self.vertical_fov,
            self.aspect_ratio,
            self.clip_near,
            self.clip_far
        );
        return perspective_projection * look_projection;
    }
}
