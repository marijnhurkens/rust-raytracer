use bvh::aabb::{Bounded, AABB};
use bvh::bounding_hierarchy::{BHShape};
use bvh::bvh::BVH;
use bvh::nalgebra::{Point3, Vector3};
use materials;
use renderer;
use std::fmt::Debug;

pub struct Scene {
    pub bg_color: Vector3<f64>,
    pub objects: Vec<Box<dyn Object>>,
    pub bvh: BVH,
    pub lights: Vec<Light>,
}

impl Scene {
    pub fn push_object(&mut self, o: Box<dyn Object>) {
        self.objects.push(o);
    }
}

pub trait Object: Debug + Send + Sync + Bounded + BHShape {
    fn get_materials(&self) -> &Vec<Box<dyn materials::Material>>;
    fn test_intersect(&self, renderer::Ray) -> Option<f64>;
    fn get_normal(&self, Point3<f64>) -> Vector3<f64>;
}

impl Bounded for Box<dyn Object> {
    fn aabb(&self) -> AABB {
        (**self).aabb()
    }
}

impl BHShape for Box<dyn Object> {
    fn set_bh_node_index(&mut self, index: usize) {
        (**self).set_bh_node_index(index);
    }

    fn bh_node_index(&self) -> usize {
        (**self).bh_node_index()
    }
}

#[derive(Debug)]
pub struct Sphere {
    pub position: Point3<f64>,
    pub radius: f64,
    pub materials: Vec<Box<dyn materials::Material>>,
    pub node_index: usize,
}

impl Object for Sphere {
    fn get_materials(&self) -> &Vec<Box<dyn materials::Material>> {
        &self.materials
    }

    fn test_intersect(&self, ray: renderer::Ray) -> Option<f64> {
        use std::f64;

        let camera_to_sphere_center = ray.point - self.position;
        let a = ray.direction.dot(&ray.direction); // camera_to_sphere length squared
        let b = camera_to_sphere_center.dot(&ray.direction);
        let c = camera_to_sphere_center.dot(&camera_to_sphere_center) - self.radius * self.radius;
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

    fn get_normal(&self, point: Point3<f64>) -> Vector3<f64> {
        (point - self.position).normalize()
    }
}

impl Bounded for Sphere {
    fn aabb(&self) -> AABB {
        let half_size = Vector3::new(self.radius, self.radius, self.radius);
        let min = self.position - half_size;
        let max = self.position + half_size;
        AABB::with_bounds(bvh::nalgebra::convert(min), bvh::nalgebra::convert(max))
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

#[derive(Debug)]
pub struct Plane {
    pub position: Point3<f64>,
    pub normal: Vector3<f64>,
    pub materials: Vec<Box<dyn materials::Material>>,
    pub node_index: usize,
}

impl Object for Plane {
    fn get_materials(&self) -> &Vec<Box<dyn materials::Material>> {
        &self.materials
    }

    fn test_intersect(&self, ray: renderer::Ray) -> Option<f64> {
        
        let denom = self.normal.dot(&ray.direction);

        if denom.abs() > 1e-4 {
            let v = self.position - ray.point;

            let distance = v.dot(&self.normal) / denom;
            
            if distance > 0.0001 {
                return Some(distance);
            }
        }

        None
    }

    fn get_normal(&self, _: Point3<f64>) -> Vector3<f64> {
        self.normal
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
        AABB::with_bounds(bvh::nalgebra::convert(min), bvh::nalgebra::convert(max))
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

#[derive(Debug, Copy, Clone)]
pub struct Light {
    pub position: Point3<f64>,
    pub intensity: f64,
    pub color: Vector3<f64>,
}
