use nalgebra::{Point3, Vector3};

// LIGHT
#[derive(Debug, Copy, Clone)]
pub struct Light {
    pub position: Point3<f64>,
    pub intensity: f64,
    pub color: Vector3<f64>,
}