use std::borrow::BorrowMut;
use std::sync::Arc;

use bvh::aabb::{Aabb, Bounded};
use bvh::bounding_hierarchy::BHShape;
use nalgebra::{Point3, Vector3};

use crate::lights::area::AreaLight;
use crate::lights::Light;
use crate::materials::Material;
use crate::objects::plane::Plane;
use crate::objects::rectangle::Rectangle;
use crate::objects::sphere::Sphere;
//use crate::objects::cube::Cube;
//use crate::objects::rectangle::Rectangle;
//use crate::objects::sphere::Sphere;
use crate::objects::triangle::Triangle;
use crate::renderer;
use crate::surface_interaction::{Interaction, SurfaceInteraction};

pub mod triangle;
pub mod sphere;
pub mod plane;
pub mod rectangle;
//pub mod cube;
//pub mod rectangle;

#[derive(Debug, Clone)]
pub enum Object {
    Sphere(Sphere),
    Triangle(Triangle),
    Plane(Plane),
    Rectangle(Rectangle),
    //Cube(Cube),
}

pub trait ObjectTrait {
    fn get_materials(&self) -> &Vec<Arc<Material>>;
    fn get_light(&self) -> Option<&Arc<Light>>;
    fn test_intersect(&self, ray: renderer::Ray) -> Option<(f64, SurfaceInteraction)>;
    fn sample_point(&self, sample: Vec<f64>) -> Interaction;
    fn pdf(&self, interaction: &Interaction, wi: Vector3<f64>) -> f64;
    fn area(&self) -> f64;
}

impl ObjectTrait for ArcObject {
    fn get_materials(&self) -> &Vec<Arc<Material>> {
        match self.0.as_ref() {
            Object::Sphere(x) => x.get_materials(),
            Object::Triangle(x) => x.get_materials(),
            Object::Plane(x) => x.get_materials(),
            Object::Rectangle(x) => x.get_materials(),
            //Object::Cube(x) => x.get_materials(),
        }
    }

    fn get_light(&self) -> Option<&Arc<Light>> {
        match self.0.as_ref() {
            Object::Sphere(x) => x.get_light(),
            Object::Triangle(x) => x.get_light(),
            Object::Plane(x) => x.get_light(),
            Object::Rectangle(x) => x.get_light(),
            //Object::Cube(x) => x.test_intersect(ray),
        }
    }

    fn test_intersect(&self, ray: renderer::Ray) -> Option<(f64, SurfaceInteraction)> {
        match self.0.as_ref() {
            Object::Sphere(x) => x.test_intersect(ray),
            Object::Triangle(x) => x.test_intersect(ray),
            Object::Plane(x) => x.test_intersect(ray),
            Object::Rectangle(x) => x.test_intersect(ray),
            //Object::Cube(x) => x.test_intersect(ray),
        }
    }

    fn sample_point(&self, sample: Vec<f64>) -> Interaction {
        match self.0.as_ref() {
            Object::Sphere(x) => x.sample_point(sample),
            Object::Triangle(x) => x.sample_point(sample),
            Object::Plane(x) => x.sample_point(sample),
            Object::Rectangle(x) => x.sample_point(sample),
            //Object::Cube(x) => x.test_intersect(ray),
        }
    }

    fn pdf(&self, interaction: &Interaction, wi: Vector3<f64>) -> f64 {
        match self.0.as_ref() {
            Object::Sphere(x) => x.pdf(interaction, wi),
            Object::Triangle(x) => x.pdf(interaction, wi),
            Object::Plane(x) => x.pdf(interaction, wi),
            Object::Rectangle(x) => x.pdf(interaction, wi),
            //Object::Cube(x) => x.test_intersect(ray),
        }
    }

    fn area(&self) -> f64 {
        match self.0.as_ref() {
            Object::Sphere(x) => x.area(),
            Object::Triangle(x) => x.area(),
            Object::Plane(x) => x.area(),
            Object::Rectangle(x) => x.area(),
            //Object::Cube(x) => x.test_intersect(ray),
        }
    }
}

#[derive(Debug)]
pub struct ArcObject(pub Arc<Object>);

impl Bounded<f32, 3> for ArcObject {
    fn aabb(&self) -> Aabb<f32, 3> {
        match self.0.as_ref() {
            Object::Sphere(x) => x.aabb(),
            Object::Triangle(x) => x.aabb(),
            Object::Plane(x) => x.aabb(),
            Object::Rectangle(x) => x.aabb(),
            //Object::Cube(x) => x.aabb(),
        }
    }
}

impl BHShape<f32, 3> for ArcObject {
    fn set_bh_node_index(&mut self, index: usize) {
        match Arc::get_mut(&mut self.0).unwrap() {
            Object::Sphere(x) => x.set_bh_node_index(index),
            Object::Triangle(x) => x.set_bh_node_index(index),
            Object::Plane(x) => x.set_bh_node_index(index),
            Object::Rectangle(x) => x.set_bh_node_index(index),
            //Object::Cube(x) => x.set_bh_node_index(index),
        }
    }

    fn bh_node_index(&self) -> usize {
        match self.0.as_ref() {
            Object::Sphere(x) => x.bh_node_index(),
            Object::Triangle(x) => x.bh_node_index(),
            Object::Plane(x) => x.bh_node_index(),
            Object::Rectangle(x) => x.bh_node_index(),
            //Object::Cube(x) => x.bh_node_index(),
        }
    }
}
