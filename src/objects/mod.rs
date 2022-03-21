use bvh::aabb::{AABB, Bounded};
use bvh::bounding_hierarchy::BHShape;
use materials::Material;
use objects::cube::Cube;
use objects::rectangle::Rectangle;
use objects::sphere::Sphere;
use objects::triangle::Triangle;
use renderer;
use surface_interaction::SurfaceInteraction;

pub mod triangle;
pub mod sphere;
pub mod plane;
pub mod cube;
pub mod rectangle;

#[derive(Debug)]
pub enum Object {
    Sphere(Sphere),
    Triangle(Triangle),
    Rectangle(Rectangle),
    Cube(Cube),
}

pub trait Objectable {
    fn get_materials(&self) -> &Vec<Material>;
    fn test_intersect(&self, ray: renderer::Ray) -> Option<(f64, SurfaceInteraction)>;
}

impl Objectable for Object {
    fn get_materials(&self) -> &Vec<Material> {
        match self {
            Object::Sphere(x) => x.get_materials(),
            Object::Triangle(x) => x.get_materials(),
            Object::Rectangle(x) => x.get_materials(),
            Object::Cube(x) => x.get_materials(),
        }
    }

    fn test_intersect(&self, ray: renderer::Ray) -> Option<(f64, SurfaceInteraction)> {
        match self {
            Object::Sphere(x) => x.test_intersect(ray),
            Object::Triangle(x) => x.test_intersect(ray),
            Object::Rectangle(x) => x.test_intersect(ray),
            Object::Cube(x) => x.test_intersect(ray),
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
            Object::Cube(x) => x.aabb(),
        }
    }
}

impl BHShape for Object {
    fn set_bh_node_index(&mut self, index: usize) {
        match self {
            Object::Sphere(x) => x.set_bh_node_index(index),
            Object::Triangle(x) => x.set_bh_node_index(index),
            Object::Rectangle(x) => x.set_bh_node_index(index),
            Object::Cube(x) => x.set_bh_node_index(index),
        }
    }

    fn bh_node_index(&self) -> usize {
        match self {
            Object::Sphere(x) => x.bh_node_index(),
            Object::Triangle(x) => x.bh_node_index(),
            Object::Rectangle(x) => x.bh_node_index(),
            Object::Cube(x) => x.bh_node_index(),
        }
    }
}