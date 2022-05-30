use std::sync::Arc;
use bvh::aabb::{Bounded, AABB};
use bvh::bounding_hierarchy::BHShape;
use nalgebra::{Point3, Vector3};
use lights::area::AreaLight;
use lights::Light;
use std::borrow::BorrowMut;

use materials::Material;
use objects::plane::Plane;
//use objects::cube::Cube;
//use objects::rectangle::Rectangle;
//use objects::sphere::Sphere;
use objects::triangle::Triangle;
use renderer;
use surface_interaction::{Interaction, SurfaceInteraction};

pub mod triangle;
//pub mod sphere;
pub mod plane;
//pub mod cube;
//pub mod rectangle;

#[derive(Debug, Clone)]
pub enum Object {
    //Sphere(Sphere),
    Triangle(Triangle),
    Plane(Plane),
    //Rectangle(Rectangle),
    //Cube(Cube),
}

pub trait ObjectTrait {
    fn get_materials(&self) -> &Vec<Material>;
    fn get_light(&self) -> Option<&Arc<Light>>;
    fn test_intersect(&self, ray: renderer::Ray) -> Option<(f64, SurfaceInteraction)>;
    fn sample_point(&self) -> Interaction;
    fn pdf(&self, interaction: &Interaction, wi: Vector3<f64>) -> f64;
    fn area(&self) -> f64;
}

impl ObjectTrait for ArcObject {
    fn get_materials(&self) -> &Vec<Material> {
        match self.0.as_ref() {
            //Object::Sphere(x) => x.get_materials(),
            Object::Triangle(x) => x.get_materials(),
            Object::Plane(x) => x.get_materials(),
            //Object::Rectangle(x) => x.get_materials(),
            //Object::Cube(x) => x.get_materials(),
        }
    }

    fn get_light(&self) -> Option<&Arc<Light>> {
        match self.0.as_ref() {
            //Object::Sphere(x) => x.test_intersect(ray),
            Object::Triangle(x) => x.get_light(),
            Object::Plane(x) => x.get_light(),
            //Object::Rectangle(x) => x.test_intersect(ray),
            //Object::Cube(x) => x.test_intersect(ray),
        }
    }

    fn test_intersect(&self, ray: renderer::Ray) -> Option<(f64, SurfaceInteraction)> {
        match self.0.as_ref() {
            //Object::Sphere(x) => x.test_intersect(ray),
            Object::Triangle(x) => x.test_intersect(ray),
            Object::Plane(x) => x.test_intersect(ray),
            //Object::Rectangle(x) => x.test_intersect(ray),
            //Object::Cube(x) => x.test_intersect(ray),
        }
    }

    fn sample_point(&self) -> Interaction {
        match self.0.as_ref() {
            //Object::Sphere(x) => x.test_intersect(ray),
            Object::Triangle(x) => x.sample_point(),
            Object::Plane(x) => x.sample_point(),
            //Object::Rectangle(x) => x.test_intersect(ray),
            //Object::Cube(x) => x.test_intersect(ray),
        }
    }

    fn pdf(&self, interaction: &Interaction, wi: Vector3<f64>) -> f64 {
        match self.0.as_ref() {
            //Object::Sphere(x) => x.test_intersect(ray),
            Object::Triangle(x) => x.pdf(interaction, wi),
            Object::Plane(x) => x.pdf(interaction, wi),
            //Object::Rectangle(x) => x.test_intersect(ray),
            //Object::Cube(x) => x.test_intersect(ray),
        }
    }

    fn area(&self) -> f64 {
        match self.0.as_ref() {
            //Object::Sphere(x) => x.test_intersect(ray),
            Object::Triangle(x) => x.area(),
            Object::Plane(x) => x.area(),
            //Object::Rectangle(x) => x.test_intersect(ray),
            //Object::Cube(x) => x.test_intersect(ray),
        }
    }
}

#[derive(Debug)]
pub struct ArcObject (pub Arc<Object>);

impl Bounded for ArcObject {
    fn aabb(&self) -> AABB {
        match self.0.as_ref() {
            //Object::Sphere(x) => x.aabb(),
            Object::Triangle(x) => x.aabb(),
            Object::Plane(x) => x.aabb(),
            //Object::Rectangle(x) => x.aabb(),
            //Object::Cube(x) => x.aabb(),
        }
    }
}

impl BHShape for ArcObject {
    fn set_bh_node_index(&mut self, index: usize) {
        match Arc::get_mut(&mut self.0).unwrap() {
            //Object::Sphere(x) => x.set_bh_node_index(index),
            Object::Triangle(x) => x.set_bh_node_index(index),
            Object::Plane(x) => x.set_bh_node_index(index),
            //Object::Rectangle(x) => x.set_bh_node_index(index),
            //Object::Cube(x) => x.set_bh_node_index(index),
        }
    }

    fn bh_node_index(&self) -> usize {
        match self.0.as_ref() {
            //Object::Sphere(x) => x.bh_node_index(),
            Object::Triangle(x) => x.bh_node_index(),
            Object::Plane(x) => x.bh_node_index(),
            //Object::Rectangle(x) => x.bh_node_index(),
            //Object::Cube(x) => x.bh_node_index(),
        }
    }
}
