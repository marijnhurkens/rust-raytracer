use cgmath::*;

#[derive(Debug)]
pub struct Camera {
    position: Vector3<f64>,
    target: Vector3<f64>,
    forward: Vector3<f64>,
    up: Vector3<f64>,
    right: Vector3<f64>,
    fov: f64,
}

impl Camera {
    pub fn new(position: Vector3<f64>, target: Vector3<f64>, fov: f64) -> Camera {
        // x left right
        // y forward backward
        // z up down

        // To find the forward vector we subtract the position from the target
        // and find the unit vector
        let forward = vec_get_unit(target - position);

        // The right vector is the dot product of forward and the z axis
        let right = vec_get_unit(Vector3::cross(forward, Vector3::unit_z()));

        // The up vector is the dot product of the right vector and the forward vector
        let up = vec_get_unit(Vector3::cross(right, forward));

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

pub fn vec_get_unit(vec: Vector3<f64>) -> Vector3<f64> {
    let magnitude = (vec.x.powf(2.0) + vec.y.powf(2.0) + vec.z.powf(2.0)).sqrt();

    vec / magnitude
}
