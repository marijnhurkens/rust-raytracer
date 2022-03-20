use std::boxed::Box as RustBox;
use std::fmt::Debug;

use bvh::aabb::{Bounded, AABB};
use bvh::bounding_hierarchy::BHShape;
use nalgebra::{Matrix3, Point2, Point3, Vector2, Vector3};

use helpers::{max_dimension_vec_3, permute};
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
    pub mesh: Arc<Mesh>,
    pub v0_index: usize,
    pub v1_index: usize,
    pub v2_index: usize,
    pub materials: Vec<RustBox<dyn materials::Material>>,
    pub node_index: usize,
}

impl Triangle {
    pub fn new(
        mesh: Arc<Mesh>,
        v0_index: usize,
        v1_index: usize,
        v2_index: usize,
        materials: Vec<RustBox<dyn materials::Material>>,
    ) -> Triangle {
        Triangle {
            mesh,
            v0_index,
            v1_index,
            v2_index,
            materials,
            node_index: 0,
        }
    }

    fn get_vertices(&self) -> (Point3<f64>, Point3<f64>, Point3<f64>) {
        (
            Point3::new(
                self.mesh.positions[3 * self.v0_index] as f64,
                self.mesh.positions[3 * self.v0_index + 1] as f64,
                self.mesh.positions[3 * self.v0_index + 2] as f64,
            ),
            Point3::new(
                self.mesh.positions[3 * self.v1_index] as f64,
                self.mesh.positions[3 * self.v1_index + 1] as f64,
                self.mesh.positions[3 * self.v1_index + 2] as f64,
            ),
            Point3::new(
                self.mesh.positions[3 * self.v2_index] as f64,
                self.mesh.positions[3 * self.v2_index + 1] as f64,
                self.mesh.positions[3 * self.v2_index + 2] as f64,
            ),
        )
    }

    fn get_normals(&self) -> (Vector3<f64>, Vector3<f64>, Vector3<f64>) {
        (
            Vector3::new(
                self.mesh.normals[3 * self.v0_index] as f64,
                self.mesh.normals[3 * self.v0_index + 1] as f64,
                self.mesh.normals[3 * self.v0_index + 2] as f64,
            ),
            Vector3::new(
                self.mesh.normals[3 * self.v1_index] as f64,
                self.mesh.normals[3 * self.v1_index + 1] as f64,
                self.mesh.normals[3 * self.v1_index + 2] as f64,
            ),
            Vector3::new(
                self.mesh.normals[3 * self.v2_index] as f64,
                self.mesh.normals[3 * self.v2_index + 1] as f64,
                self.mesh.normals[3 * self.v2_index + 2] as f64,
            ),
        )
    }
}

impl Triangle {
    fn get_materials(&self) -> &Vec<RustBox<dyn materials::Material>> {
        &self.materials
    }
    fn test_intersect(&self, ray: renderer::Ray) -> Option<Intersection> {
        let (p0, p1, p2) = self.get_vertices();

        let mut p0t = p0 - ray.point;
        let mut p1t = p1 - ray.point;
        let mut p2t = p2 - ray.point;

        let kz = max_dimension_vec_3(ray.direction.abs());
        let kx = (kz + 1) % 3;
        let ky = (kx + 1) % 3;

        let d = permute(ray.direction, kx, ky, kz);
        p0t = permute(p0t, kx, ky, kz);
        p1t = permute(p1t, kx, ky, kz);
        p2t = permute(p2t, kx, ky, kz);

        let s_x = -d.x / d.z;
        let s_y = -d.y / d.z;
        let s_z = 1.0 / d.z;
        p0t.x += s_x * p0t.z;
        p0t.y += s_y * p0t.z;
        p1t.x += s_x * p1t.z;
        p1t.y += s_y * p1t.z;
        p2t.x += s_x * p2t.z;
        p2t.y += s_y * p2t.z;

        let e0 = p1t.x * p2t.y - p1t.y * p2t.x;
        let e1 = p2t.x * p0t.y - p2t.y * p0t.x;
        let e2 = p0t.x * p1t.y - p0t.y * p1t.x;

        if (e0 < 0.0 || e1 < 0.0 || e2 < 0.0) && (e0 > 0.0 || e1 > 0.0 || e2 > 0.0) {
            return None;
        }

        let det = e0 + e1 + e2;
        if det == 0.0 {
            return None;
        }

        p0t.z *= s_z;
        p1t.z *= s_z;
        p2t.z *= s_z;
        let t_scaled = e0 * p0t.z + e1 * p1t.z + e2 * p2t.z;
        // todo: implement ray.t_max instead of 1000.0
        if det < 0.0 && (t_scaled >= 0.0 || t_scaled < 1000.0 * det) {
            return None;
        } else if det > 0.0 && (t_scaled <= 0.0 || t_scaled > 1000.0 * det) {
            return None;
        }

        let inv_det = 1.0 / det;
        let b0 = e0 * inv_det;
        let b1 = e1 * inv_det;
        let b2 = e2 * inv_det;
        let t = t_scaled * inv_det;

        if t > f64::EPSILON {
            let uv = vec![
                Point2::new(0.0, 0.0),
                Point2::new(1.0, 0.0),
                Point2::new(1.0, 1.0),
            ];

            let duv02: Vector2<f64> = uv[0] - uv[2];
            let duv12: Vector2<f64> = uv[1] - uv[2];
            let dp02: Vector3<f64> = p0 - p2;
            let dp12: Vector3<f64> = p1 - p2;

            let determinant = duv02.x * duv12.y - duv02.y * duv12.x;
            let invdet = 1.0 / determinant;
            //let u = (duv12.y * dp02 - duv02.y * dp12) * invdet;
            //let v = (-duv12.x * dp02 + duv02.x * dp12) * invdet;

            let (p0_normal, p1_normal, p2_normal) = self.get_normals();
            let normal = (b0 * p0_normal + b1 * p1_normal + b2 * p2_normal).normalize();

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
        let (p0, p1, p2) = self.get_vertices();

        let min_x = p0.x.min(p1.x.min(p2.x));
        let min_y = p0.y.min(p1.y.min(p2.y));
        let min_z = p0.z.min(p1.z.min(p2.z));
        let max_x = p0.x.max(p1.x.max(p2.x));
        let max_y = p0.y.max(p1.y.max(p2.y));
        let max_z = p0.z.max(p1.z.max(p2.z));

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
