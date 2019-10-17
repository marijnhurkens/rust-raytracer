use bvh::bvh::BVH;
use bvh::nalgebra::{Vector3};

pub mod sceneLoader;
pub mod objects;
pub mod lights;
pub mod materials;

pub struct Scene {
    pub bg_color: Vector3<f64>,
    pub objects: Vec<Box<dyn objects::Object>>,
    pub bvh: BVH,
    pub lights: Vec<lights::Light>,
}

impl Scene {
    pub fn push_object(&mut self, o: Box<dyn objects::Object>) {
        self.objects.push(o);
    }
}