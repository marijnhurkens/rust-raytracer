use nalgebra::{Point3, Vector3};

use lights::area::AreaLight;
use lights::distant::DistantLight;
use lights::point::PointLight;
use renderer::Ray;
use surface_interaction::{Interaction, SurfaceInteraction};

pub mod area;
pub mod point;
pub mod distant;

#[derive(Debug)]
pub enum Light {
    Point(PointLight),
    Area(AreaLight),
    Distant(DistantLight),
}

pub trait LightTrait {
    fn is_delta(&self) -> bool;
    fn sample_irradiance(&self, interaction: &SurfaceInteraction) -> LightIrradianceSample;
    fn sample_emitting(&self) -> LightEmittingSample;
    fn pdf_incidence(&self, interaction: &Interaction, wi: Vector3<f64>) -> f64;
    fn pdf_emitting(&self, ray: Ray, light_normal: Vector3<f64>) -> LightEmittingPdf;
    fn power(&self) -> Vector3<f64>;
}

pub struct LightIrradianceSample {
    pub irradiance: Vector3<f64>,
    pub point: Point3<f64>,
    pub wi: Vector3<f64>,
    pub pdf: f64,
}

pub struct LightEmittingSample {
    pub ray: Ray,
    pub light_normal: Vector3<f64>,
    pub pdf_position: f64,
    pub pdf_direction: f64,
}

pub struct LightEmittingPdf {
    pub pdf_position: f64,
    pub pdf_direction: f64,
}

impl LightTrait for Light {
    fn is_delta(&self) -> bool {
        match self {
            Light::Point(x) => x.is_delta(),
            Light::Area(x) => x.is_delta(),
            Light::Distant(x) => x.is_delta(),
        }
    }

    /// Sample_Li()
    fn sample_irradiance(&self, interaction: &SurfaceInteraction) -> LightIrradianceSample {
        match self {
            Light::Point(x) => x.sample_irradiance(interaction),
            Light::Area(x) => x.sample_irradiance(interaction),
            Light::Distant(x) => x.sample_irradiance(interaction),
        }
    }

    /// Sample_le()
    fn sample_emitting(&self) -> LightEmittingSample {
        match self {
            Light::Point(x) => x.sample_emitting(),
            Light::Area(x) => x.sample_emitting(),
            Light::Distant(x) => x.sample_emitting(),
        }
    }

    /// Pdf_li()
    fn pdf_incidence(&self, interaction: &Interaction, wi: Vector3<f64>) -> f64 {
        match self {
            Light::Point(x) => x.pdf_incidence(interaction, wi),
            Light::Area(x) => x.pdf_incidence(interaction, wi),
            Light::Distant(x) => x.pdf_incidence(interaction, wi),
        }
    }

    /// Pdf_Le()
    fn pdf_emitting(&self, ray: Ray, light_normal: Vector3<f64>) -> LightEmittingPdf {
        match self {
            Light::Point(x) => x.pdf_emitting(ray, light_normal),
            Light::Area(x) => x.pdf_emitting(ray, light_normal),
            Light::Distant(x) => x.pdf_emitting(ray, light_normal),
        }
    }

    fn power(&self) -> Vector3<f64> {
        match self {
            Light::Point(x) => x.power(),
            Light::Area(x) => x.power(),
            Light::Distant(x) => x.power(),
        }
    }
}
