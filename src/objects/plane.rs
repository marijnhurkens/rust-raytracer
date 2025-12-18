use std::sync::Arc;

use bvh::aabb::{Aabb, Bounded};
use bvh::bounding_hierarchy::BHShape;
use nalgebra::{Point3, Vector2, Vector3};

use crate::helpers::coordinate_system;
use crate::lights::Light;
use crate::materials::Material;
use crate::objects::ObjectTrait;
use crate::renderer;
use crate::renderer::{
    debug_write_pixel_f64, debug_write_pixel_f64_on_bounce, debug_write_pixel_on_bounce, Ray,
};
use crate::surface_interaction::{Interaction, SurfaceInteraction};

#[derive(Debug, Clone)]
pub struct Plane {
    pub position: Point3<f64>,
    pub normal: Vector3<f64>,
    pub materials: Vec<Arc<Material>>,
    pub node_index: usize,
}

impl Plane {
    pub fn new(position: Point3<f64>, normal: Vector3<f64>, materials: Vec<Arc<Material>>) -> Self {
        Plane {
            position,
            normal,
            materials,
            node_index: 0,
        }
    }
}

impl ObjectTrait for Plane {
    fn get_materials(&self) -> &Vec<Arc<Material>> {
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

        // todo: fix?
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

    fn sample_point(&self, _: Vec<f64>) -> Interaction {
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

impl Bounded<f32, 3> for Plane {
    fn aabb(&self) -> Aabb<f32, 3> {
        use std::f64;

        // The plane is infinite
        const MAX_SIZE: f64 = 1e14;

        let half_size = Vector3::new(MAX_SIZE, MAX_SIZE, MAX_SIZE);
        let min = self.position - half_size;
        let max = self.position + half_size;

        Aabb::with_bounds(
            Point3::new(min.x as f32, min.y as f32, min.z as f32),
            Point3::new(max.x as f32, max.y as f32, max.z as f32),
        )
    }
}

impl BHShape<f32, 3> for Plane {
    fn set_bh_node_index(&mut self, index: usize) {
        self.node_index = index;
    }

    fn bh_node_index(&self) -> usize {
        self.node_index
    }
}
