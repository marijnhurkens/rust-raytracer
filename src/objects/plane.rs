use std::sync::Arc;

use bvh::aabb::{Bounded, AABB};
use bvh::bounding_hierarchy::BHShape;
use nalgebra::{Point3, Vector2, Vector3};

use helpers::coordinate_system;
use lights::Light;
use materials::Material;
use objects::ObjectTrait;
use renderer;
use renderer::{
    debug_write_pixel_f64, debug_write_pixel_f64_on_bounce, debug_write_pixel_on_bounce, Ray,
};
use surface_interaction::{Interaction, SurfaceInteraction};

#[derive(Debug, Clone)]
pub struct Plane {
    pub position: Point3<f64>,
    pub normal: Vector3<f64>,
    pub materials: Vec<Material>,
    pub node_index: usize,
}

impl Plane {
    pub fn new(position: Point3<f64>, normal: Vector3<f64>, materials: Vec<Material>) -> Self {
        Plane {
            position,
            normal,
            materials,
            node_index: 0,
        }
    }
}

impl ObjectTrait for Plane {
    fn get_materials(&self) -> &Vec<Material> {
        &self.materials
    }

    fn get_light(&self) -> Option<&Arc<Light>> {
        None
    }

    fn test_intersect(&self, ray: renderer::Ray) -> Option<(f64, SurfaceInteraction)> {
        let denom = self.normal.dot(&ray.direction);

        if denom.abs() < 1e-9 {
            return None;
        }

        let v = self.position - ray.point;
        let distance = v.dot(&self.normal) / denom;

        if distance < 0.0000001 {
            return None;
        }

        let p_hit = ray.point + ray.direction * distance + self.normal * 1e-9;
        //let (sn, ss, ts) = coordinate_system(self.normal);

        let ss = Vector3::new(1.0, -0.0, 0.0);
        let ts = Vector3::new(0.0, 0.0, -1.0);

        Some((
            distance,
            SurfaceInteraction::new(
                p_hit,
                self.normal,
                -ray.direction,
                Vector2::zeros(),
                ss,
                ts,
                Vector3::repeat(10000.0),
                Vector3::repeat(1000.0),
                Vector3::zeros(),
            ),
        ))
    }

    fn sample_point(&self) -> Interaction {
        unimplemented!();
    }

    fn pdf(&self, interaction: &Interaction, wi: Vector3<f64>) -> f64 {
        unimplemented!();
        // let ray = Ray {
        //     point: interaction.point + wi * 1e-9,
        //     direction: wi,
        // };
        //
        // let intersect_object = self.test_intersect(ray);
        //
        // if intersect_object.is_none() {
        //     return 0.0;
        // }
        //
        // let (_, surface_interaction) = intersect_object.unwrap();
        //
        // (interaction.point - surface_interaction.point).magnitude_squared()
        //     / (surface_interaction.shading_normal.dot(&-wi).abs() * self.area())
    }

    fn area(&self) -> f64 {
        unimplemented!();
    }
}

impl Bounded for Plane {
    fn aabb(&self) -> AABB {
        use std::f64;

        // The plane is infinite
        const MAX_SIZE: f64 = 1e14;

        let half_size = Vector3::new(MAX_SIZE, MAX_SIZE, MAX_SIZE);
        let min = self.position - half_size;
        let max = self.position + half_size;

        AABB::with_bounds(
            bvh::Point3::new(min.x as f32, min.y as f32, min.z as f32),
            bvh::Point3::new(max.x as f32, max.y as f32, max.z as f32),
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
