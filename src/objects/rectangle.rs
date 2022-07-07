use std::sync::Arc;

use bvh::aabb::{Bounded, AABB};
use bvh::bounding_hierarchy::BHShape;
use nalgebra::{Matrix3, Point3, SimdPartialOrd, Vector2, Vector3};

use crate::helpers::coordinate_system;
use crate::lights::Light;
use crate::materials::Material;
use crate::objects::ObjectTrait;
use crate::renderer;
use crate::renderer::{debug_write_pixel, debug_write_pixel_f64, Ray};
use crate::surface_interaction::{Interaction, SurfaceInteraction};

// RECTANGLE
#[derive(Debug, Clone)]
pub struct Rectangle {
    pub position: Point3<f64>,
    pub side_a: Vector3<f64>,
    pub side_b: Vector3<f64>,
    pub materials: Vec<Material>,
    pub light: Option<Arc<Light>>,
    pub node_index: usize,
}

impl Rectangle {
    pub fn new(
        position: Point3<f64>,
        side_a: Vector3<f64>,
        side_b: Vector3<f64>,
        materials: Vec<Material>,
        light: Option<Arc<Light>>,
    ) -> Self {
        Rectangle {
            position,
            side_a,
            side_b,
            materials,
            light,
            node_index: 0,
        }
    }

    fn get_normal(&self) -> Vector3<f64> {
        self.side_a.cross(&self.side_b).normalize()
    }
}

impl ObjectTrait for Rectangle {
    fn get_materials(&self) -> &Vec<Material> {
        &self.materials
    }

    fn get_light(&self) -> Option<&Arc<Light>> {
        self.light.as_ref()
    }

    fn test_intersect(&self, ray: renderer::Ray) -> Option<(f64, SurfaceInteraction)> {
        let normal = self.get_normal();
        let denom = normal.dot(&ray.direction);

        if denom.abs() < 1e-9 {
            return None;
        }

        let v = self.position - ray.point;
        let distance = v.dot(&normal) / denom;

        if distance < 1e-9 {
            return None;
        }

        // point on intersection plane
        let p = ray.point + (ray.direction * distance);

        let p0p = p - self.position;

        let a = p0p.dot(&self.side_a) / self.side_a.dot(&self.side_a);
        let b = p0p.dot(&self.side_b) / self.side_b.dot(&self.side_b);

        if !(0.0..=1.0).contains(&a) || !(0.0..=1.0).contains(&b) {
            return None;
        }

        let (sn, ss, ts) = coordinate_system(normal);

        Some((
            distance,
            SurfaceInteraction::new(
                p,
                normal,
                -ray.direction,
                Vector2::zeros(),
                ss,
                ts,
                ss,
                ts,
                Vector3::zeros(),
            ),
        ))
    }

    fn sample_point(&self, sample: Vec<f64>) -> Interaction {
        let point = self.position + (self.side_a * sample[0]) + (self.side_b * sample[1]);

        Interaction {
            point,
            normal: self.get_normal(),
        }
    }

    // todo: duplicate code with triangle
    fn pdf(&self, interaction: &Interaction, wi: Vector3<f64>) -> f64 {
        let ray = Ray {
            point: interaction.point + wi * 1e-9,
            direction: wi,
        };

        let intersect_object = self.test_intersect(ray);

        if intersect_object.is_none() {
            return 0.0;
        }

        let (_, surface_interaction) = intersect_object.unwrap();

        nalgebra::distance_squared(&interaction.point, &surface_interaction.point)
            / (surface_interaction.shading_normal.dot(&-wi).abs() * self.area())
    }

    fn area(&self) -> f64 {
        self.side_b.magnitude() * self.side_b.magnitude()
    }
}

impl Bounded for Rectangle {
    fn aabb(&self) -> AABB {
        let pos_opposite = self.position + self.side_a + self.side_b;
        let min = self.position.simd_min(pos_opposite);
        let max = self.position.simd_max(pos_opposite);

        AABB::with_bounds(
            bvh::Point3::new(min.x as f32, min.y as f32, min.z as f32),
            bvh::Point3::new(max.x as f32, max.y as f32, max.z as f32),
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
