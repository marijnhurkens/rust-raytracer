use std::f64::consts::PI;

use nalgebra::{distance_squared, Point3, Vector3};

use surface_interaction::SurfaceInteraction;

pub enum Light {
    PointLight(PointLight),
}

pub trait LightTrait {
    fn is_delta(&self) -> bool;
    fn get_position(&self) -> Point3<f64>;
    fn sample_irradiance(&self, interaction: &SurfaceInteraction) -> Vector3<f64>;
    fn power(&self) -> Vector3<f64>;
}

impl LightTrait for Light {
    fn is_delta(&self) -> bool {
        match self {
            Light::PointLight(x) => x.is_delta(),
        }
    }

    fn get_position(&self) -> Point3<f64> {
        match self {
            Light::PointLight(x) => x.get_position(),
        }
    }

    fn sample_irradiance(&self, interaction: &SurfaceInteraction) -> Vector3<f64> {
        match self {
            Light::PointLight(x) => x.sample_irradiance(interaction),
        }
    }

    fn power(&self) -> Vector3<f64> {
        match self {
            Light::PointLight(x) => x.power(),
        }
    }
}

pub struct PointLight {
    position: Point3<f64>,
    intensity: Vector3<f64>,
}

impl LightTrait for PointLight {
    fn is_delta(&self) -> bool {
        true
    }

    fn get_position(&self) -> Point3<f64> {
        self.position
    }

    fn sample_irradiance(&self, interaction: &SurfaceInteraction) -> Vector3<f64> {
        self.intensity / distance_squared(&self.position, &interaction.point)
    }

    fn power(&self) -> Vector3<f64> {
        4.0 * PI * self.intensity
    }
}

impl PointLight {
    pub fn new(position: Point3<f64>, intensity: Vector3<f64>) -> Self {
        Self {
            position,
            intensity,
        }
    }
}
