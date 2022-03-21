use std::fmt::Debug;

use bvh::aabb::{Bounded, AABB};
use bvh::bounding_hierarchy::BHShape;
use nalgebra::{Point3, Vector3};

use materials::Material;
use renderer::{Ray};
use surface_interaction::SurfaceInteraction;

#[derive(Debug)]
pub struct Cube {
    pub position: Point3<f64>,
    pub width: f64,
    pub height: f64,
    pub rotation: Vector3<f64>,
    pub node_index: usize,
    pub materials: Vec<Material>,
}

impl Cube {
    pub fn get_materials(&self) -> &Vec<Material> {
        &self.materials
    }

    pub fn test_intersect(&self, _renderer: Ray) -> Option<(f64, SurfaceInteraction)> {
        todo!()
    }
}

impl Bounded for Cube {
    fn aabb(&self) -> AABB {
        todo!()
    }
}

impl BHShape for Cube {
    fn set_bh_node_index(&mut self, index: usize) {
        self.node_index = index;
    }

    fn bh_node_index(&self) -> usize {
        self.node_index
    }
}
