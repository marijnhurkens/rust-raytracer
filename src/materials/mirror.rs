use nalgebra::Vector3;

use crate::bsdf::helpers::fresnel::{Fresnel, FresnelNoop};
use crate::bsdf::specular_reflection::SpecularReflection;
use crate::bsdf::{Bsdf, BXDF};
use crate::materials::MaterialTrait;
use crate::surface_interaction::SurfaceInteraction;

#[derive(Debug, Clone, PartialEq)]
pub struct MirrorMaterial {
    pub reflectance_color: Vector3<f64>,
}

impl MirrorMaterial {
    pub fn new(reflectance_color: Vector3<f64>) -> Self {
        MirrorMaterial { reflectance_color }
    }
}

impl MaterialTrait for MirrorMaterial {
    fn compute_scattering_functions(&self, si: &mut SurfaceInteraction) {
        let mut bsdf = Bsdf::new(*si, None);

        bsdf.add(BXDF::SpecularReflection(SpecularReflection::new(
            self.reflectance_color,
            Fresnel::Noop(FresnelNoop::new()),
        )));

        si.bsdf = Some(bsdf);
    }

    fn get_albedo(&self) -> Vector3<f64> {
        self.reflectance_color
    }
}
