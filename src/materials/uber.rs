use nalgebra::Vector3;
use num_traits::Zero;

use crate::bsdf::helpers::fresnel::{Fresnel, FresnelDielectric};
use crate::bsdf::helpers::microfacet_distribution::{
    MicrofacetDistribution, TrowbridgeReitzDistribution,
};
use crate::bsdf::lambertian::Lambertian;
use crate::bsdf::microfacet_reflection::MicrofacetReflection;
use crate::bsdf::microfacet_transmission::MicrofacetTransmission;
use crate::bsdf::oren_nayar::OrenNayar;
use crate::bsdf::specular_reflection::SpecularReflection;
use crate::bsdf::specular_transmission::{SpecularTransmission, TransportMode};
use crate::bsdf::{Bsdf, Bxdf};
use crate::materials::MaterialTrait;
use crate::surface_interaction::SurfaceInteraction;

#[derive(Debug, Clone, PartialEq)]
pub struct UberMaterial {
    // base diffuse color
    diffuse: Vector3<f64>,
    // specular highlight color
    specular: Vector3<f64>,
    // transmission factor
    transmission: Vector3<f64>,
    roughness: f64,
    ior: f64,
}

impl UberMaterial {
    pub fn new(
        diffuse: Vector3<f64>,
        specular: Vector3<f64>,
        transmission: Vector3<f64>,
        roughness: f64,
        ior: f64,
    ) -> Self {
        UberMaterial {
            diffuse,
            specular,
            transmission,
            roughness,
            ior,
        }
    }
}

impl MaterialTrait for UberMaterial {
    fn compute_scattering_functions(&self, si: &mut SurfaceInteraction) {
        let mut bsdf = si.bsdf.unwrap_or(Bsdf::new(*si, None));

        if !self.diffuse.is_zero() {
            bsdf.add(Bxdf::Lambertian(Lambertian::new(self.diffuse)));
        }

        if !self.specular.is_zero() {
            let fresnel = FresnelDielectric::new(1.0, self.ior);

            if self.roughness < 1.0e-3 {
                bsdf.add(Bxdf::SpecularReflection(SpecularReflection::new(
                    self.specular,
                    Fresnel::Dielectric(fresnel),
                )));
            } else {
                let roughness = TrowbridgeReitzDistribution::roughness_to_alpha(self.roughness);
                let distribution = TrowbridgeReitzDistribution::new(roughness, roughness, true);

                bsdf.add(Bxdf::MicrofacetReflection(MicrofacetReflection::new(
                    self.specular,
                    distribution,
                    fresnel,
                )));
            }
        }

        if !self.transmission.is_zero() {
            let fresnel = FresnelDielectric::new(1.0, self.ior);

            if self.roughness < 1.0e-3 {
                bsdf.add(Bxdf::SpecularTransmission(SpecularTransmission::new(
                    self.transmission,
                    1.0,
                    self.ior,
                    TransportMode::Radiance,
                )));
            } else {
                let roughness = TrowbridgeReitzDistribution::roughness_to_alpha(self.roughness);
                let distribution = TrowbridgeReitzDistribution::new(roughness, roughness, true);

                bsdf.add(Bxdf::MicrofacetTransmission(MicrofacetTransmission::new(
                    self.transmission,
                    distribution,
                    fresnel,
                    1.0,
                    self.ior,
                )));
            }
        }

        si.bsdf = Some(bsdf);
    }

    fn get_albedo(&self) -> Vector3<f64> {
        self.diffuse
    }
}
