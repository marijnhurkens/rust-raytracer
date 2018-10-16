use cgmath::*;
use image::Rgba;

#[derive(Debug,Clone)]
pub struct Scene {
    pub bg_color: Vector3<f64>,
    pub spheres: Vec<Sphere>,
    pub lights: Vec<Light>,
}

#[derive(Debug,Copy,Clone)]
pub struct Sphere {
    pub position: Vector3<f64>,
    pub radius: f64,
    pub color: Vector3<f64>,
    pub lambert: f64,
    pub specular: f64,
    pub ambient: f64,
}

#[derive(Debug,Copy,Clone)]
pub struct Light {
    pub position: Vector3<f64>,
    pub intensity: f64,
    pub color: Vector3<f64>,
}