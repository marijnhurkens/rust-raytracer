use bvh::bvh::BVH;
use nalgebra::{Vector3};
use serde::{Serialize, Deserialize};

pub mod scene_loader;
pub mod objects;
pub mod lights;
pub mod materials;
pub mod camera;

#[derive(Serialize, Deserialize)]
pub struct Scene {
    pub bg_color: Vector3<f64>,
    
    pub objects: Vec<Box<dyn objects::Object>>,
    
    #[serde(skip, default="get_default_bvh")]
    pub bvh: BVH,
    
    pub lights: Vec<lights::Light>,
}

fn get_default_bvh() -> BVH {
    let mut objects: Vec<Box<dyn objects::Object>> = vec![];

    BVH::build(&mut objects)
}

impl Scene {
    pub fn push_object(&mut self, o: Box<dyn objects::Object>) {
        self.objects.push(o);
    }
}