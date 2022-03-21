use core::f64;
use bvh::aabb::{Bounded, AABB};
use bvh::bounding_hierarchy::BHShape;
use nalgebra::{Point3, Vector2, Vector3};

use materials::Material;
use renderer;
use surface_interaction::SurfaceInteraction;

// SPHERE
#[derive(Debug)]
pub struct Sphere {
    pub position: Point3<f64>,
    pub radius: f64,
    pub materials: Vec<Box<dyn Material>>,
    pub node_index: usize,
}

impl Sphere {
    fn get_normal(&self, point: Point3<f64>) -> Vector3<f64> {
        (point - self.position).normalize()
    }

    pub fn get_materials(&self) -> &Vec<Box<dyn Material>> {
        &self.materials
    }

    pub fn test_intersect(&self, ray: renderer::Ray) -> Option<(f64, SurfaceInteraction)> {
        use std::f64;

        let ray_to_sphere_center = ray.point - self.position;
        let a = ray.direction.dot(&ray.direction); // camera_to_sphere length squared
        let b = ray_to_sphere_center.dot(&ray.direction);
        let c = ray_to_sphere_center.dot(&ray_to_sphere_center) - self.radius * self.radius;
        let discriminant = b * b - a * c;

        if discriminant < 0.0 {
            return None;
        }

        let temp_dist = (-b - (b * b - a * c).sqrt()) / a;

        if temp_dist > 0.0001 && temp_dist < f64::MAX {
            let contact_point = ray.point + ray.direction * temp_dist;
            let normal = self.get_normal(contact_point);

            return Some((
                temp_dist,
                SurfaceInteraction {
                    point: ray.point + ray.direction * temp_dist,
                    surface_normal: normal,
                    wo: -ray.direction,
                    uv: Vector2::zeros(),
                },
            ));
        }

        let temp_dist = (-b + (b * b - a * c).sqrt()) / a;

        if temp_dist > 0.0001 && temp_dist < f64::MAX {
            let contact_point = ray.point + ray.direction * temp_dist;
            let normal = self.get_normal(contact_point);

            return Some((
                temp_dist,
                SurfaceInteraction {
                    point: ray.point + ray.direction * temp_dist,
                    surface_normal: normal,
                    wo: -ray.direction,
                    uv: Vector2::zeros(),
                },
            ));
        }

        None
    }
}

impl Bounded for Sphere {
    fn aabb(&self) -> AABB {
        let half_size = Vector3::new(self.radius, self.radius, self.radius);
        let min = self.position - half_size;
        let max = self.position + half_size;

        AABB::with_bounds(
            bvh::nalgebra::Point3::new(min.x as f32, min.y as f32, min.z as f32),
            bvh::nalgebra::Point3::new(max.x as f32, max.y as f32, max.z as f32),
        )
    }
}

impl BHShape for Sphere {
    fn set_bh_node_index(&mut self, index: usize) {
        self.node_index = index;
    }

    fn bh_node_index(&self) -> usize {
        self.node_index
    }
}
