use cgmath::*;
use image::Rgba;

#[derive(Debug,Clone)]
pub struct Scene {
    pub bg_color: Rgba<u8>,
    pub spheres: Vec<Sphere>,
}

#[derive(Debug,Copy,Clone)]
pub struct Sphere {
    pub position: Vector3<f64>,
    pub radius: f64,

}