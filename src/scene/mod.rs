use bvh::bvh::BVH;
use nalgebra::{Vector3};


pub mod objects;
pub mod lights;
pub mod materials;
pub mod camera;

pub struct Scene {
    pub bg_color: Vector3<f64>,
    pub objects: Vec<Box<dyn objects::Object>>,
    pub bvh: BVH,
    pub lights: Vec<lights::Light>,
}

impl Scene {
    pub fn new(bg_color: Vector3<f64>,  lights: Vec<lights::Light>, objects: Vec<Box<dyn objects::Object>>, bvh: BVH)->Scene {
       
        Scene {
            bg_color: bg_color,
            objects: objects,
            lights: lights,
            bvh: bvh
        }
    }

    pub fn push_object(&mut self, o: Box<dyn objects::Object>) {
        self.objects.push(o);
    }
}