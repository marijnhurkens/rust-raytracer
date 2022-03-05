use std::boxed::Box as RustBox;
use std::fmt::Debug;

use bvh::aabb::{Bounded, AABB};
use bvh::bounding_hierarchy::BHShape;
use nalgebra::{Matrix3, Point3, Vector3};

use materials::Material;
use renderer::{Intersection, Ray};

use crate::renderer;

use super::*;

#[derive(Debug)]
pub enum Object {
    Sphere(Sphere),
    Triangle(Triangle),
    Rectangle(Rectangle),
    Box(Box),
}

pub trait Objectable {
    fn get_materials(&self) -> &Vec<RustBox<dyn materials::Material>>;
    fn test_intersect(&self, ray: renderer::Ray) -> Option<Intersection>;
}

impl Objectable for Object {
    fn get_materials(&self) -> &Vec<RustBox<dyn materials::Material>> {
        match self {
            Object::Sphere(x) => x.get_materials(),
            Object::Triangle(x) => x.get_materials(),
            Object::Rectangle(x) => x.get_materials(),
            Object::Box(x) => x.get_materials(),
        }
    }

    fn test_intersect(&self, ray: renderer::Ray) -> Option<Intersection> {
        match self {
            Object::Sphere(x) => x.test_intersect(ray),
            Object::Triangle(x) => x.test_intersect(ray),
            Object::Rectangle(x) => x.test_intersect(ray),
            Object::Box(x) => x.test_intersect(ray),
        }
    }
}

// BOX
impl Bounded for Object {
    fn aabb(&self) -> AABB {
        match self {
            Object::Sphere(x) => x.aabb(),
            Object::Triangle(x) => x.aabb(),
            Object::Rectangle(x) => x.aabb(),
            Object::Box(x) => x.aabb(),
        }
    }
}

impl BHShape for Object {
    fn set_bh_node_index(&mut self, index: usize) {
        match self {
            Object::Sphere(x) => x.set_bh_node_index(index),
            Object::Triangle(x) => x.set_bh_node_index(index),
            Object::Rectangle(x) => x.set_bh_node_index(index),
            Object::Box(x) => x.set_bh_node_index(index),
        }
    }

    fn bh_node_index(&self) -> usize {
        match self {
            Object::Sphere(x) => x.bh_node_index(),
            Object::Triangle(x) => x.bh_node_index(),
            Object::Rectangle(x) => x.bh_node_index(),
            Object::Box(x) => x.bh_node_index(),
        }
    }
}

// SPHERE
#[derive(Debug)]
pub struct Sphere {
    pub position: Point3<f64>,
    pub radius: f64,
    pub materials: Vec<RustBox<dyn materials::Material>>,
    pub node_index: usize,
}

impl Sphere {
    fn get_normal(&self, point: Point3<f64>) -> Vector3<f64> {
        (point - self.position).normalize()
    }

    fn get_materials(&self) -> &Vec<RustBox<dyn materials::Material>> {
        &self.materials
    }

    fn test_intersect(&self, ray: crate::renderer::Ray) -> Option<Intersection> {
        use std::f64;

        let ray_to_sphere_center = ray.point - self.position;
        let a = ray.direction.dot(&ray.direction); // camera_to_sphere length squared
        let b = ray_to_sphere_center.dot(&ray.direction);
        let c = ray_to_sphere_center.dot(&ray_to_sphere_center) - self.radius * self.radius;
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

        if temp_dist > 0.0001 && temp_dist < f64::MAX {
            let contact_point = ray.point + ray.direction * temp_dist;
            let n = &self.get_normal(contact_point);

            return Some(Intersection {
                distance: temp_dist,
                normal: *n,
            });
        }

        let temp_dist = (-b + (b * b - a * c).sqrt()) / a;

        if temp_dist > 0.0001 && temp_dist < f64::MAX {
            let contact_point = ray.point + ray.direction * temp_dist;
            let n = &self.get_normal(contact_point);

            return Some(Intersection {
                distance: temp_dist,
                normal: *n,
            });
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

// BOX
#[derive(Debug)]
pub struct Box {
    pub position: Point3<f64>,
    pub width: f64,
    pub height: f64,
    pub rotation: Vector3<f64>,
    pub node_index: usize,
    pub materials: Vec<RustBox<dyn materials::Material>>,
}

impl Box {
    fn get_materials(&self) -> &Vec<RustBox<dyn Material>> {
        &self.materials
    }

    fn test_intersect(&self, _renderer: Ray) -> Option<Intersection> {
        todo!()
    }
}

impl Bounded for Box {
    fn aabb(&self) -> AABB {
        todo!()
    }
}

impl BHShape for Box {
    fn set_bh_node_index(&mut self, index: usize) {
        self.node_index = index;
    }

    fn bh_node_index(&self) -> usize {
        self.node_index
    }
}

// PLANE
#[derive(Debug)]
pub struct Plane {
    pub position: Point3<f64>,
    pub normal: Vector3<f64>,
    pub materials: Vec<RustBox<dyn materials::Material>>,
    pub node_index: usize,
}

impl Plane {
    fn get_materials(&self) -> &Vec<RustBox<dyn materials::Material>> {
        &self.materials
    }

    fn test_intersect(&self, ray: renderer::Ray) -> Option<Intersection> {
        let denom = self.normal.dot(&ray.direction);

        if denom.abs() > 1e-4 {
            let v = self.position - ray.point;

            let distance = v.dot(&self.normal) / denom;

            if distance > 0.0001 {
                return Some(Intersection {
                    distance,
                    normal: self.normal,
                });
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

// RECTANGLE
#[derive(Debug)]
pub struct Rectangle {
    pub position: Point3<f64>,
    pub side_a: Vector3<f64>,
    pub side_b: Vector3<f64>,
    pub materials: Vec<RustBox<dyn materials::Material>>,
    pub node_index: usize,
}

impl Rectangle {
    fn get_normal(&self) -> Vector3<f64> {
        self.side_a.cross(&self.side_b)
    }
}

impl Rectangle {
    fn get_materials(&self) -> &Vec<RustBox<dyn materials::Material>> {
        &self.materials
    }

    fn test_intersect(&self, ray: renderer::Ray) -> Option<Intersection> {
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
            //dbg!(self.get_normal());
            return Some(Intersection { distance, normal });
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

// Triangle
#[derive(Debug)]
pub struct Triangle {
    pub v0: Point3<f64>,
    pub v1: Point3<f64>,
    pub v2: Point3<f64>,
    v0_normal: Vector3<f64>,
    v1_normal: Vector3<f64>,
    v2_normal: Vector3<f64>,
    pub materials: Vec<RustBox<dyn materials::Material>>,
    pub node_index: usize,
}

impl Triangle {
    pub fn new(
        v0: Point3<f64>,
        v1: Point3<f64>,
        v2: Point3<f64>,
        v0_normal: Vector3<f64>,
        v1_normal: Vector3<f64>,
        v2_normal: Vector3<f64>,
        materials: Vec<RustBox<dyn materials::Material>>,
    ) -> Triangle {
        Triangle {
            v0,
            v1,
            v2,
            v0_normal,
            v1_normal,
            v2_normal,
            materials,
            node_index: 0,
        }
    }
}

impl Triangle {
    fn get_materials(&self) -> &Vec<RustBox<dyn materials::Material>> {
        &self.materials
    }
    fn test_intersect(&self, ray: renderer::Ray) -> Option<Intersection> {
        let v0v1 = self.v1 - self.v0;
        let v0v2 = self.v2 - self.v0;

        let u_vec = ray.direction.cross(&v0v2);
        let det = v0v1.dot(&u_vec);

        if det < f64::EPSILON {
            return None;
        }

        let inv_det = 1.0 / det;

        let a_to_origin = ray.point - self.v0;
        let u = a_to_origin.dot(&u_vec) * inv_det;

        if !(0.0..=1.0).contains(&u) {
            return None;
        }

        let v_vec = a_to_origin.cross(&v0v1);
        let v = ray.direction.dot(&v_vec) * inv_det;
        if v < 0.0 || (u + v) > 1.0 {
            return None;
        }

        let t = v0v2.dot(&v_vec) * inv_det;

        if t > f64::EPSILON {
            let normal = u * self.v1_normal + v * self.v2_normal + (1.0 - u - v) * self.v0_normal;

            return Some(Intersection {
                distance: t,
                normal,
            });
        }

        None
    }
}

impl Bounded for Triangle {
    fn aabb(&self) -> AABB {
        let min_x = self.v0.x.min(self.v1.x.min(self.v2.x));
        let min_y = self.v0.y.min(self.v1.y.min(self.v2.y));
        let min_z = self.v0.z.min(self.v1.z.min(self.v2.z));
        let max_x = self.v0.x.max(self.v1.x.max(self.v2.x));
        let max_y = self.v0.y.max(self.v1.y.max(self.v2.y));
        let max_z = self.v0.z.max(self.v1.z.max(self.v2.z));

        AABB::with_bounds(
            bvh::nalgebra::Point3::new(min_x as f32, min_y as f32, min_z as f32),
            bvh::nalgebra::Point3::new(
                (max_x + 0.001) as f32,
                (max_y + 0.001) as f32,
                (max_z + 0.001) as f32,
            ),
        )
    }
}

impl BHShape for Triangle {
    fn set_bh_node_index(&mut self, index: usize) {
        self.node_index = index;
    }

    fn bh_node_index(&self) -> usize {
        self.node_index
    }
}
