use cgmath::*;
use materials;

#[derive(Debug)]
pub struct Scene {
    pub bg_color: Vector3<f64>,
    pub spheres: Vec<Sphere>,
    pub lights: Vec<Light>,
}

#[derive(Debug)]
pub struct Sphere {
    pub position: Vector3<f64>,
    pub radius: f64,
    pub materials: Vec<Box<materials::Material>>,
}


#[derive(Debug,Copy,Clone)]
pub struct Light {
    pub position: Vector3<f64>,
    pub intensity: f64,
    pub color: Vector3<f64>,
}