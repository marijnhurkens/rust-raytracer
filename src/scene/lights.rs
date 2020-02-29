use nalgebra::{Point3, Vector3};
use serde::{Serialize, Deserialize};

// LIGHT
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Light {
    pub position: Point3<f64>,
    pub intensity: f64,
    pub color: Vector3<f64>,
}