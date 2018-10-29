use cgmath::*;
use materials;
use renderer;
use std::fmt::Debug;
use bvh::aabb::{AABB, Bounded};

#[derive(Debug)]
pub struct Scene {
    pub bg_color: Vector3<f64>,
    pub spheres: Vec<Sphere>,
    pub objects: Vec<Box<Object>>,
    pub lights: Vec<Light>,
}

impl Scene {
    pub fn push_object(&mut self, o: Box<dyn Object>) {
        self.objects.push(o);
    }
}

pub trait Object: Debug + Send + Sync {
    fn get_materials(&self) -> &Vec<Box<materials::Material>>;
    fn test_intersect(&self, renderer::Ray) -> Option<f64>;
    fn get_normal(&self, Vector3<f64>) -> Vector3<f64>;
}

#[derive(Debug)]
pub struct Sphere {
    pub position: Vector3<f64>,
    pub radius: f64,
    pub materials: Vec<Box<materials::Material>>,
}

impl Object for Sphere {
    fn get_materials(&self) -> &Vec<Box<materials::Material>> {
        &self.materials
    }

    fn test_intersect(&self, ray: renderer::Ray) -> Option<f64> {
        use std::f64;

        let camera_to_sphere_center = ray.point - self.position;
        let a = ray.direction.dot(ray.direction); // camera_to_sphere length squared
        let b = camera_to_sphere_center.dot(ray.direction);
        let c = camera_to_sphere_center.dot(camera_to_sphere_center) - self.radius * self.radius;
        let discriminant = b * b - a * c;

        // let v = camera_to_sphere_center.dot(ray.direction);
        // let eo_dot = camera_to_sphere_center.dot(camera_to_sphere_center); // camera_to_sphere length squared
        // let discriminant = sphere.radius.powf(2.0) - eo_dot + v.powf(2.0); // radius - camera_to_sphere_center length + v all squared

        // Example:
        //
        // ray starts from 0,0,0 with direction 0,0,1 (forward and a bit up)
        // sphere is as 0,4,0 with radius 1
        //
        // camera_to_sphere_center = 0,4,0 - 0,0,0 = 0,4,0
        // v = camera_to_sphere_center . ray.direction = 0*0 + 4*0 + 0*1 = 0
        // eoDot = camera_to_sphere_center . itself = 0*0 + 4*4 + 0*0 = 16
        // discriminant = sphere radius ^ 2 - eoDot + v ^ 2 = (1*1) - 16 + (0*0) = -15
        // no hit

        //discriminant = (sphere.radius * sphere.radius) - eoDot + (v * v);

        // Some(1.0)
        if discriminant < 0.0 {
            return None;
        }

        let temp_dist = (-b - (b * b - a * c).sqrt()) / a;

        //return Some(temp_dist);

        if temp_dist > 0.0001 && temp_dist < f64::MAX {
            return Some(temp_dist);
        }

        let temp_dist = (-b + (b * b - a * c).sqrt()) / a;

        if temp_dist > 0.0001 && temp_dist < f64::MAX {
            return Some(temp_dist);
        }
        None
    }

    fn get_normal(&self, point: Vector3<f64>) -> Vector3<f64> {
        (point - self.position).normalize()
    }
}


#[derive(Debug)]
pub struct Plane {
    pub position: Vector3<f64>,
    pub normal: Vector3<f64>,
    pub materials: Vec<Box<materials::Material>>,
}

impl Object for Plane {
    fn get_materials(&self) -> &Vec<Box<materials::Material>> {
        &self.materials
    }

    fn test_intersect(&self, ray: renderer::Ray) -> Option<f64> {
        let denom = self.normal.dot(ray.direction);

        if denom.abs() > 1e-6 {
            let v = self.position - ray.point;

            let distance = v.dot(self.normal) / denom;

            if distance >= 0.0 {
                return Some(distance);
            }
        }

        None
    }

    fn get_normal(&self, _: Vector3<f64>) -> Vector3<f64> {
        self.normal
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Light {
    pub position: Vector3<f64>,
    pub intensity: f64,
    pub color: Vector3<f64>,
}
