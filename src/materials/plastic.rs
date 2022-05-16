use nalgebra::Vector3;

use bsdf::fresnel::DielectricFresnel;
use bsdf::lambertian::Lambertian;
use bsdf::specular_reflection::SpecularReflection;
use bsdf::{BSDF, BXDF};
use materials::MaterialTrait;
use surface_interaction::SurfaceInteraction;
use num_traits::Zero;


#[derive(Debug, Clone)]
pub struct PlasticMaterial {
    pub diffuse: Vector3<f64>,
    pub specular: Vector3<f64>,
    pub roughness: f64,
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
        let mut bsdf = BSDF::new(*si, None);

        if !self.diffuse.is_zero() {
            bsdf.add(BXDF::Lambertian(Lambertian::new(self.diffuse)));
        }

        if !self.specular.is_zero() {
            let fresnel = DielectricFresnel::new(1.0, 1.5);
            bsdf.add(BXDF::SpecularReflection(SpecularReflection::new(
                self.specular,
                fresnel,
            )));
        }

        si.bsdf = Some(bsdf);
    }
}
