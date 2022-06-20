use bvh::aabb::{Bounded, AABB};
use bvh::bounding_hierarchy::BHShape;
use nalgebra::{Matrix3, Point3, Vector2, Vector3};

use crate::materials::Material;
use crate::renderer;
use crate::surface_interaction::SurfaceInteraction;

// RECTANGLE
#[derive(Debug)]
pub struct Rectangle {
    pub position: Point3<f64>,
    pub side_a: Vector3<f64>,
    pub side_b: Vector3<f64>,
    pub materials: Vec<Material>,
    pub node_index: usize,
}

impl Rectangle {
    fn get_normal(&self) -> Vector3<f64> {
        self.side_a.cross(&self.side_b)
    }
}

impl Rectangle {
    pub fn get_materials(&self) -> &Vec<Material> {
        &self.materials
    }

    pub fn test_intersect(&self, ray: renderer::Ray) -> Option<(f64, SurfaceInteraction)> {
        let normal = self.get_normal();
        let denom = normal.dot(&ray.direction);

        if denom.abs() < 1e-4 {
            return None;
        }

        let v = self.position - ray.point;

        let distance = v.dot(&normal) / denom;

        if distance < 0.0001 {
            return None;
        }

        //println!("test");
        // point on intersection plane
        let p = ray.point + (ray.direction * distance);

        let a = self.position;
        let b = self.position + self.side_a;
        let c = self.position + self.side_a + self.side_b;
        let d = self.position + self.side_b;

        let m1 = a - b;
        let m2 = c - b;
        let m3 = m1.cross(&m2);
        let m = Matrix3::from_columns(&[m1, m2, m3]).try_inverse().unwrap();

        let x_transformed = m * (d - b);
        let p_transformed = m * (p - b);

        if p_transformed.x > 0.0
            && p_transformed.y > 0.0
            && p_transformed.x < x_transformed.x
            && p_transformed.y < x_transformed.y
        {
            return Some((
                distance,
                SurfaceInteraction::new(
                    ray.point + ray.direction * distance,
                    normal,
                    -ray.direction,
                    Vector2::zeros(),
                ),
            ));
        }

        None
    }
}

impl Bounded for Rectangle {
    fn aabb(&self) -> AABB {
        let pos_opposite = self.position + self.side_a + self.side_b;
        let min_x = self.position.x.min(pos_opposite.x);
        let min_y = self.position.y.min(pos_opposite.y);
        let min_z = self.position.z.min(pos_opposite.z);
        let max_x = self.position.x.max(pos_opposite.x);
        let max_y = self.position.y.max(pos_opposite.y);
        let max_z = self.position.z.max(pos_opposite.z);

        AABB::with_bounds(
            bvh::nalgebra::Point3::new(min_x as f32, min_y as f32, min_z as f32),
            bvh::nalgebra::Point3::new(max_x as f32, max_y as f32, max_z as f32),
        )
    }
}

impl BHShape for Rectangle {
    fn set_bh_node_index(&mut self, index: usize) {
        self.node_index = index;
    }

    fn bh_node_index(&self) -> usize {
        self.node_index
    }
}
