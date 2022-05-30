use std::f64::consts::PI;
use std::fmt::DebugSet;
use nalgebra::{distance_squared, Point3};
use nalgebra::Vector3;
use lights::{LightEmittingPdf, LightEmittingSample, LightIrradianceSample, LightTrait};
use renderer::Ray;
use surface_interaction::{Interaction, SurfaceInteraction};

#[derive(Debug)]
pub struct DistantLight {
    world_center: Point3<f64>,
    world_radius: f64,
    direction: Vector3<f64>,
    intensity: Vector3<f64>,
}

impl LightTrait for DistantLight {
    fn is_delta(&self) -> bool {
        true
    }

    // Sample_Li
    fn sample_irradiance(&self, interaction: &SurfaceInteraction) -> LightIrradianceSample {
        let wi = self.direction;
        let pdf = 1.0;

        let point_outside = interaction.point + self.direction * (2.0 * self.world_radius);

        LightIrradianceSample {
            point: point_outside,
            wi,
            pdf,
            irradiance: self.intensity,
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
        unimplemented!();
    }

    fn power(&self) -> Vector3<f64> {
        self.world_radius * self.world_radius * PI * self.intensity
    }
}

impl DistantLight {
    pub fn new(world_center: Point3<f64>, world_radius: f64, direction: Vector3<f64>, intensity: Vector3<f64>) -> Self {
        Self {
            world_center,
            world_radius,
            direction: direction.normalize(),
            intensity,
        }
    }
}
