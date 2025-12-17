use crate::helpers::{coordinate_system, gamma, get_random_in_unit_sphere};
use crate::lights::Light;
use crate::materials::Material;
use crate::objects::ObjectTrait;
use crate::renderer;
use crate::surface_interaction::{Interaction, SurfaceInteraction};
use bvh::aabb::{Aabb, Bounded};
use bvh::bounding_hierarchy::BHShape;
use core::f64;
use nalgebra::{Matrix, Point3, Vector2, Vector3};
use std::sync::Arc;

// SPHERE
#[derive(Debug, Clone)]
pub struct Sphere {
    pub position: Point3<f64>,
    pub radius: f64,
    pub materials: Vec<Material>,
    pub node_index: usize,
}

impl Sphere {
    pub fn new(position: Point3<f64>, radius: f64, materials: Vec<Material>) -> Self {
        Sphere {
            position,
            radius,
            materials,
            node_index: 0,
        }
    }

    fn get_normal(&self, point: Point3<f64>) -> Vector3<f64> {
        (point - self.position).normalize()
    }

    fn interaction_from_intersection(
        &self,
        point: Point3<f64>,
        normal: Vector3<f64>,
        wo: Vector3<f64>,
    ) -> SurfaceInteraction {
        let object_point = point - self.position;

        let mut phi = object_point.y.atan2(object_point.x);
        if phi < 0.0 {
            phi += 2.0 * f64::consts::PI;
        }

        let u = phi / (2.0 * f64::consts::PI);
        let cos_theta = (object_point.z / self.radius).clamp(-1.0, 1.0);
        let theta = cos_theta.acos();
        let v = theta / f64::consts::PI;

        // compute dp/du and dp/dv
        // let z_radius = (point.x * point.x + point.y * point.y).sqrt();
        // let cos_phi = point.x / z_radius;
        // let sin_phi = point.y / z_radius;
        let sin_phi = phi.sin();
        let cos_phi = phi.cos();
        let sin_theta = theta.sin();

        let dp_du = Vector3::new(
            -2.0 * f64::consts::PI * self.radius * sin_theta * sin_phi,
            2.0 * f64::consts::PI * self.radius * sin_theta * cos_phi,
            0.0,
        );
        let dp_dv = Vector3::new(
            self.radius * cos_theta * cos_phi,
            self.radius * cos_theta * sin_phi,
            -self.radius * sin_theta,
        ) * f64::consts::PI;

        // compute dndu and dndv
        let dn_du = -2.0 * f64::consts::PI * Vector3::new(point.x, point.y, 0.0);
        let dn_dv =
            -f64::consts::PI * Vector3::new(cos_theta * cos_phi, cos_theta * sin_phi, -sin_theta);

        let p_error = Vector3::zeros();//gamma(5.0) * Vector3::new(point.x, point.y, point.z).abs();

        let (ss, ts) = {
            let mut ss = dp_du.normalize();
            let mut ts = normal.cross(&ss);
            if ts.magnitude_squared() > 0.0 {
                ts = ts.normalize();
                ss = ts.cross(&normal);
                (ss, ts)
            } else {
                let (_, ss, ts) = coordinate_system(normal);
                (ss, ts)
            }
        };


        SurfaceInteraction::new(
            point + normal * 1e-9,
            normal,
            wo,
            Vector2::new(u, v),
            ss,
            ts,
            dp_du,
            dp_dv,
            p_error,
        )
    }
}

impl ObjectTrait for Sphere {
    fn get_materials(&self) -> &Vec<Material> {
        &self.materials
    }

    fn get_light(&self) -> Option<&Arc<Light>> {
        None
    }

    fn test_intersect(&self, ray: renderer::Ray) -> Option<(f64, SurfaceInteraction)> {
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
                self.interaction_from_intersection(contact_point, normal, -ray.direction),
            ));
        }

        let temp_dist = (-b + (b * b - a * c).sqrt()) / a;

        if temp_dist > 0.0001 && temp_dist < f64::MAX {
            let contact_point = ray.point + ray.direction * temp_dist;
            let normal = self.get_normal(contact_point);

            return Some((
                temp_dist,
                self.interaction_from_intersection(contact_point, normal, -ray.direction),
            ));
        }

        None
    }

    fn sample_point(&self, sample: Vec<f64>) -> Interaction {
        unimplemented!()
    }

    fn pdf(&self, interaction: &Interaction, wi: Vector3<f64>) -> f64 {

       1.0 / self.area()
    }

    fn area(&self) -> f64 {
        4.0 * f64::consts::PI * self.radius * self.radius
    }
}

impl Bounded<f32, 3> for Sphere {
    fn aabb(&self) -> Aabb<f32, 3> {
        let half_size = Vector3::new(self.radius, self.radius, self.radius);
        let min = self.position - half_size;
        let max = self.position + half_size;

        Aabb::with_bounds(
            Point3::new(min.x as f32, min.y as f32, min.z as f32),
            Point3::new(max.x as f32, max.y as f32, max.z as f32),
        )
    }
}

impl BHShape<f32, 3> for Sphere {
    fn set_bh_node_index(&mut self, index: usize) {
        self.node_index = index;
    }

    fn bh_node_index(&self) -> usize {
        self.node_index
    }
}
