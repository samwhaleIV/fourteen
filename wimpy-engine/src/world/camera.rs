use glam::*;
use std::f32::consts::{TAU,FRAC_PI_2};

const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.0001;

/// Basic camera controller suitable for free cam or FPS controller
/// 
/// Uses the Blender right hand coordinate system, positive Z up
pub struct WimpyCamera {
    /// Eye point
    position: Vec3,
    /// Yaw in radians (look left and right)
    yaw: f32,
    /// Pitch in radians (look up and down)
    pitch: f32,

    /// The computed angle/direction normal, in sync with yaw and pitch
    angle: Vec3,

    position_mat: Mat4,
}

impl Default for WimpyCamera {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            yaw: 0.0,
            pitch: 0.0,
            position_mat: Mat4::IDENTITY,
            angle: Vec3::new(0.0,1.0,0.0),
        }
    }
}

pub enum CameraPositionStrategy {
    /// The camera tranposes its own eye position, based on the previous position
    /// 
    /// Useful for debugging purposes
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
    /// The camera eye positioned by a third party
    /// 
    /// Useful if the camera is attached to a physics
    /// constrained object like a player controller
    Manual {
        /// Origin for the camera 'eye' point, appropriately and obviously named 'eye'
        eye: Vec3
    }
}

pub struct CameraPerspectivePacket {
    /// Vertical field of view, in degrees
    /// 
    /// `90` degrees is a reasonable starting point
    pub fov: f32,
    /// The near clipping distance of the camera view frustum
    /// 
    /// Good starting point at `0.025`
    pub clip_near: f32,

    /// The far clipping distance of the camera view frustum
    /// 
    /// Good starting point at `100.0`
    pub clip_far: f32,

    /// The aspect ratio of the viewport, expressed as `w / h`
    pub aspect_ratio: f32,
}

pub struct CameraPositionUpdate {
    /// Strategy for modifying the camera's current eye position
    pub position: CameraPositionStrategy,

    /// Time between displayed/executed frames in seconds
    pub delta_seconds: f32,

    /// Yaw look angle (left and right rotation)
    /// 
    /// Degrees (NOT per second)
    pub yaw_delta: f32,

    /// Pitch look angle (up and down rotation)
    /// 
    /// Degrees (NOT per second)
    pub pitch_delta: f32,
}

fn get_eye_angle(yaw: f32,pitch: f32) -> Vec3 {
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
        forward.y,
        -forward.x,
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

impl WimpyCamera {

    pub fn update_position(&mut self,packet: CameraPositionUpdate) {

        self.yaw += packet.yaw_delta.to_radians();
        self.pitch += packet.pitch_delta.to_radians();

        self.yaw %= TAU;
        if self.yaw < 0.0 {
            self.yaw += TAU
        }

        self.pitch = self.pitch.clamp(-PITCH_LIMIT,PITCH_LIMIT);
        /*
            Angle of eye needs to be updated BEFORE the reposition of the eye -
            keep this caveat in mind when implementing an external position controller
        */

        self.angle = get_eye_angle(self.yaw,self.pitch);

        self.position = match packet.position {
            CameraPositionStrategy::FreeCam {
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
                    self.position,
                    self.angle,
                    movement_delta,
                    packet.delta_seconds
                )
            },
            CameraPositionStrategy::Manual { eye } => eye,
        };

        self.position_mat = Mat4::look_to_rh(
            self.position,
            self.angle,
            Vec3::Z
        );
    }

    pub fn get_matrix(&self,packet: CameraPerspectivePacket) -> Mat4 {
        let perspective_mat = Mat4::perspective_rh(
            packet.fov.to_radians(),
            packet.aspect_ratio,
            packet.clip_near,
            packet.clip_far
        );
        perspective_mat * self.position_mat
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }
}
