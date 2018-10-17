use cgmath::*;

#[derive(Debug, Copy, Clone)]
pub struct Camera {
    pub position: Vector3<f64>,
    pub target: Vector3<f64>,
    pub forward: Vector3<f64>,
    pub up: Vector3<f64>,
    pub right: Vector3<f64>,
    pub fov: f64,
}

impl Camera {
    pub fn new(position: Vector3<f64>, target: Vector3<f64>, fov: f64) -> Camera {
        // x left right
        // y forward backward
        // z up down

        // To find the forward vector we subtract the position from the target
        // and find the unit vector
        let forward = (target - position).normalize();

        // The right vector is the dot product of forward and the z axis
        let right = Vector3::cross(forward, Vector3::unit_y()).normalize();

        // The up vector is the dot product of the right vector and the forward vector
        let up = Vector3::cross(right, forward).normalize();

        Camera {
            position: position,
            target: target,
            forward: forward,
            right: right,
            up: up,
            fov: fov,
        }
    }
}
