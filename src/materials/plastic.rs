use nalgebra::Vector3;
use num_traits::Zero;

use crate::bsdf::helpers::fresnel::FresnelDielectric;
use crate::bsdf::helpers::microfacet_distribution::{
    MicrofacetDistribution, TrowbridgeReitzDistribution,
};
use crate::bsdf::lambertian::Lambertian;
use crate::bsdf::microfacet_reflection::MicrofacetReflection;
use crate::bsdf::specular_reflection::SpecularReflection;
use crate::bsdf::{Bsdf, Bxdf};
use crate::materials::MaterialTrait;
use crate::surface_interaction::SurfaceInteraction;

#[derive(Debug, Clone, PartialEq)]
pub struct PlasticMaterial {
    diffuse: Vector3<f64>,
    specular: Vector3<f64>,
    roughness: f64,
}

impl PlasticMaterial {
    pub fn new(diffuse: Vector3<f64>, specular: Vector3<f64>, roughness: f64) -> Self {
        PlasticMaterial {
            diffuse,
            specular,
            roughness,
        }
    }
}

impl MaterialTrait for PlasticMaterial {
    fn compute_scattering_functions(&self, si: &mut SurfaceInteraction) {
        let mut bsdf = Bsdf::new(*si, None);

        if !self.diffuse.is_zero() {
            bsdf.add(Bxdf::Lambertian(Lambertian::new(self.diffuse)));
        }

        // todo: bug in microfacets, creates spots
        if !self.specular.is_zero() {
            let fresnel = FresnelDielectric::new(1.0, 1.5);
            let roughness = TrowbridgeReitzDistribution::roughness_to_alpha(self.roughness);
            let distribution = TrowbridgeReitzDistribution::new(roughness, roughness, true);
            //
            // bsdf.add(BXDF::SpecularReflection(SpecularReflection::new(
            //     self.specular,
            //     fresnel,
            // )));
            bsdf.add(Bxdf::MicrofacetReflection(MicrofacetReflection::new(
                self.specular,
                distribution,
                fresnel,
            )));
        }

        si.bsdf = Some(bsdf);
    }

    fn get_albedo(&self) -> Vector3<f64> {
        self.diffuse
    }
}
