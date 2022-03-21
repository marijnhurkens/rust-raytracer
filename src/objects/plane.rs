use bvh::aabb::{Bounded, AABB};
use bvh::bounding_hierarchy::BHShape;
use nalgebra::{Point3, Vector2, Vector3};

use materials::Material;
use renderer;
use surface_interaction::SurfaceInteraction;

#[derive(Debug)]
pub struct Plane {
    pub position: Point3<f64>,
    pub normal: Vector3<f64>,
    pub materials: Vec<Box<dyn Material>>,
    pub node_index: usize,
}

impl Plane {
    pub fn get_materials(&self) -> &Vec<Box<dyn Material>> {
        &self.materials
    }

    pub fn test_intersect(&self, ray: renderer::Ray) -> Option<(f64, SurfaceInteraction)> {
        let denom = self.normal.dot(&ray.direction);

        if denom.abs() > 1e-4 {
            let v = self.position - ray.point;

            let distance = v.dot(&self.normal) / denom;
            let p_hit = ray.point + ray.direction * distance;

            if distance > 0.0001 {
                return Some((
                    distance,
                    SurfaceInteraction {
                        point: p_hit,
                        surface_normal: self.normal,
                        wo: -ray.direction,
                        uv: Vector2::zeros(),
                    },
                ));
            }
        }

        None
    }
}

impl Bounded for Plane {
    fn aabb(&self) -> AABB {
        use std::f64;

        // The plane is infinite
        const MAX_SIZE: f64 = 1000000000.0;

        let half_size = Vector3::new(MAX_SIZE, MAX_SIZE, MAX_SIZE);
        let min = self.position - half_size;
        let max = self.position + half_size;

        AABB::with_bounds(
            bvh::nalgebra::Point3::new(min.x as f32, min.y as f32, min.z as f32),
            bvh::nalgebra::Point3::new(max.x as f32, max.y as f32, max.z as f32),
        )
    }
}

impl BHShape for Plane {
    fn set_bh_node_index(&mut self, index: usize) {
        self.node_index = index;
    }

    fn bh_node_index(&self) -> usize {
        self.node_index
    }
}
