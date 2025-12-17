use nalgebra::Vector3;

use crate::bsdf::helpers::fresnel::{Fresnel, FresnelNoop};
use crate::bsdf::specular_reflection::SpecularReflection;
use crate::bsdf::specular_transmission::{SpecularTransmission, TransportMode};
use crate::bsdf::{Bsdf, Bxdf};
use crate::materials::MaterialTrait;
use crate::surface_interaction::SurfaceInteraction;

#[derive(Debug, Clone, PartialEq)]
pub struct GlassMaterial {
    refraction_color: Vector3<f64>,
}

impl GlassMaterial {
    pub fn new(refraction_color: Vector3<f64>) -> Self {
        GlassMaterial { refraction_color }
    }
}

impl MaterialTrait for GlassMaterial {
    fn compute_scattering_functions(&self, si: &mut SurfaceInteraction) {
        let mut bsdf = si.bsdf.unwrap_or(Bsdf::new(*si, None));

        bsdf.add(Bxdf::SpecularTransmission(SpecularTransmission::new(
            self.refraction_color,
            1.0,
            1.5,
            TransportMode::Radiance,
        )));

        si.bsdf = Some(bsdf);
    }

    fn get_albedo(&self) -> Vector3<f64> {
        Vector3::repeat(0.0)
    }
}
