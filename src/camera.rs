use nalgebra::{Point3, Vector3};

#[derive(Debug, Copy, Clone)]
pub struct Camera {
    pub position: Point3<f64>,
    pub target: Point3<f64>,
    pub forward: Vector3<f64>,
    pub up: Vector3<f64>,
    pub right: Vector3<f64>,
    pub fov: f64,
}

impl Camera {
    pub fn new(position: Point3<f64>, target: Point3<f64>, fov: f64) -> Camera {
        // x +right -left
        // y +up -down
        // z +backward -forward

        // To find the forward vector we subtract the position from the target
        // and find the unit vector
        let forward = (target - position).normalize();

        // The right vector is the dot product of forward and a unit y vector
        let right = forward.cross(&Vector3::new(0.0,1.0,0.0)).normalize();

        // The up vector is the dot product of the right vector and the forward vector
        let up = right.cross(&forward).normalize();

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
