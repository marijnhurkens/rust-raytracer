use std::f64::consts::PI;
use std::sync::Arc;

use nalgebra::Vector3;

use lights::{LightEmittingPdf, LightEmittingSample, LightIrradianceSample, LightTrait};
use Object;
use objects::{ArcObject, ObjectTrait};
use renderer::{debug_write_pixel_f64, Ray};
use surface_interaction::{Interaction, SurfaceInteraction};

#[derive(Debug)]
pub struct AreaLight {
    object: ArcObject,
    intensity: Vector3<f64>,
}

impl LightTrait for AreaLight {
    fn is_delta(&self) -> bool {
        false
    }

    /// Sample_Li()
    fn sample_irradiance(&self, interaction: &SurfaceInteraction) -> LightIrradianceSample {
        let light_interaction = self.object.sample_point();
        let wi = (light_interaction.point - interaction.point).normalize();
        let pdf = 1.0 / self.object.area();
        let irradiance = self.irradiance_at_point(&light_interaction, -wi);

        LightIrradianceSample {
            point: light_interaction.point,
            wi,
            pdf,
            irradiance
        }
    }

    // Sample_Le()
    fn sample_emitting(&self) -> LightEmittingSample {
        todo!()
    }


    // Pdf_Li()
    fn pdf_incidence(&self, interaction: &Interaction, wi: Vector3<f64>) -> f64 {
        self.object.pdf(interaction, wi)
    }

    // Pdf_Le()
    fn pdf_emitting(&self, ray: Ray, light_normal: Vector3<f64>) -> LightEmittingPdf {
        LightEmittingPdf {
            pdf_direction: 0.0,
            pdf_position: 0.0,
        }
    }

    fn power(&self) -> Vector3<f64> {
        self.intensity * self.area() * PI
    }


}

impl AreaLight {
    pub fn new(object: ArcObject, intensity: Vector3<f64>) -> Self {
        Self {
            object,
            intensity,
        }
    }

    fn area(&self) -> f64
    {
        self.object.area()
    }

    /// L()
    pub fn irradiance_at_point(&self, interaction: &Interaction, wo: Vector3<f64>) -> Vector3<f64>
    {
        if interaction.normal.dot(&wo) > 0.0 {
            self.intensity
        } else {
            Vector3::zeros()
        }
    }

}
