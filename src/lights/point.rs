use std::f64::consts::PI;

use nalgebra::Vector3;
use nalgebra::{distance_squared, Point3};

use crate::lights::{LightEmittingPdf, LightEmittingSample, LightIrradianceSample, LightTrait};
use crate::renderer::Ray;
use crate::surface_interaction::{Interaction, SurfaceInteraction};

#[derive(Debug)]
pub struct PointLight {
    position: Point3<f64>,
    intensity: Vector3<f64>,
}

impl LightTrait for PointLight {
    fn is_delta(&self) -> bool {
        true
    }

    fn emitting(&self, interaction: &SurfaceInteraction, w: Vector3<f64>) -> Vector3<f64> {
        unimplemented!();
    }

    // Sample_Li
    fn sample_irradiance(
        &self,
        interaction: &SurfaceInteraction,
        _: Vec<f64>,
    ) -> LightIrradianceSample {
        let wi = (self.get_position() - interaction.point).normalize();
        let pdf = 1.0;
        let irradiance = self.intensity / distance_squared(&self.position, &interaction.point);

        LightIrradianceSample {
            point: self.get_position(),
            wi,
            pdf,
            irradiance,
        }
    }

    // Sample_Le()
    fn sample_emitting(&self) -> LightEmittingSample {
        unimplemented!()
    }

    // Pdf_Li()
    fn pdf_incidence(&self, interaction: &Interaction, wi: Vector3<f64>) -> f64 {
        unimplemented!()
    }

    // Pdf_Le()
    fn pdf_emitting(&self, ray: Ray, light_normal: Vector3<f64>) -> LightEmittingPdf {
        LightEmittingPdf {
            pdf_position: 0.0,
            pdf_direction: 1.0 / (PI.powi(4)),
        }
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

    fn get_position(&self) -> Point3<f64> {
        self.position
    }
}
