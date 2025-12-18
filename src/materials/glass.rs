use nalgebra::{Reflection, Vector3};

use crate::bsdf::helpers::fresnel::{Fresnel, FresnelDielectric, FresnelNoop};
use crate::bsdf::specular_reflection::SpecularReflection;
use crate::bsdf::specular_transmission::{SpecularTransmission, TransportMode};
use crate::bsdf::{Bsdf, Bxdf};
use crate::materials::MaterialTrait;
use crate::surface_interaction::SurfaceInteraction;

#[derive(Debug, Clone, PartialEq)]
pub struct GlassMaterial {
    ior: f64,
    reflection_color: Vector3<f64>,
    refraction_color: Vector3<f64>,
}

impl GlassMaterial {
    pub fn new(
        ior: f64,
        reflection_color: Vector3<f64>,
        refraction_color: Vector3<f64>,
    ) -> Self {
        GlassMaterial {
            ior,
            reflection_color,
            refraction_color,
        }
    }
}

impl MaterialTrait for GlassMaterial {
    fn compute_scattering_functions(&self, si: &mut SurfaceInteraction) {
        let mut bsdf = si.bsdf.unwrap_or(Bsdf::new(*si, None));

        bsdf.add(Bxdf::SpecularTransmission(SpecularTransmission::new(
            self.refraction_color,
            1.0,
            self.ior,
            TransportMode::Radiance,
        )));

        bsdf.add(Bxdf::SpecularReflection(SpecularReflection::new(
            self.reflection_color,
            Fresnel::Dielectric(FresnelDielectric::new(1.0, self.ior)),
        )));

        si.bsdf = Some(bsdf);
    }

    fn get_albedo(&self) -> Vector3<f64> {
        Vector3::repeat(0.0)
    }
}
