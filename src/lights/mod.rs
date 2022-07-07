use nalgebra::{Point3, Vector2, Vector3};

use crate::lights::area::AreaLight;
use crate::lights::distant::DistantLight;
use crate::lights::point::PointLight;
use crate::renderer::Ray;
use crate::surface_interaction::{Interaction, SurfaceInteraction};

pub mod area;
pub mod distant;
pub mod point;

#[derive(Debug)]
pub enum Light {
    Point(PointLight),
    Area(AreaLight),
    Distant(DistantLight),
}

pub trait LightTrait {
    fn is_delta(&self) -> bool;

    // L()
    fn emitting(&self, interaction: &SurfaceInteraction, w: Vector3<f64>) -> Vector3<f64>;

    // Sample_Li()
    fn sample_irradiance(
        &self,
        interaction: &SurfaceInteraction,
        sample: Vec<f64>,
    ) -> LightIrradianceSample;

    // Sample_Le()
    fn sample_emitting(&self) -> LightEmittingSample;

    // Pdf_Li()
    fn pdf_incidence(&self, interaction: &Interaction, wi: Vector3<f64>) -> f64;

    // Pdf_Le()
    fn pdf_emitting(&self, ray: Ray, light_normal: Vector3<f64>) -> LightEmittingPdf;

    fn environment_emitting(&self, ray: Ray) -> Vector3<f64>;
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

    fn emitting(&self, interaction: &SurfaceInteraction, w: Vector3<f64>) -> Vector3<f64> {
        match self {
            Light::Point(x) => x.emitting(interaction, w),
            Light::Area(x) => x.emitting(interaction, w),
            Light::Distant(x) => x.emitting(interaction, w),
        }
    }

    /// Sample_Li()
    fn sample_irradiance(
        &self,
        interaction: &SurfaceInteraction,
        sample: Vec<f64>,
    ) -> LightIrradianceSample {
        match self {
            Light::Point(x) => x.sample_irradiance(interaction, sample),
            Light::Area(x) => x.sample_irradiance(interaction, sample),
            Light::Distant(x) => x.sample_irradiance(interaction, sample),
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

    // Le()
    fn environment_emitting(&self, ray: Ray) -> Vector3<f64> {
        match self {
            Light::Point(x) => Vector3::zeros(),
            Light::Area(x) => Vector3::zeros(),
            Light::Distant(x) => x.environment_emitting(ray),
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
