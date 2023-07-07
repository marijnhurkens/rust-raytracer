use nalgebra::Vector3;
use num_traits::Zero;

use crate::bsdf::lambertian::Lambertian;
use crate::bsdf::oren_nayar::OrenNayar;
use crate::bsdf::{Bsdf, Bxdf};
use crate::materials::MaterialTrait;
use crate::surface_interaction::SurfaceInteraction;

#[derive(Debug, Clone, PartialEq)]
pub struct MatteMaterial {
    reflectance_color: Vector3<f64>,
    roughness: f64,
}

impl MatteMaterial {
    pub fn new(reflectance_color: Vector3<f64>, roughness: f64) -> Self {
        MatteMaterial {
            reflectance_color,
            roughness,
        }
    }
}

impl MaterialTrait for MatteMaterial {
    fn compute_scattering_functions(&self, si: &mut SurfaceInteraction) {
        let mut bsdf = Bsdf::new(*si, None);
        let sigma = self.roughness.clamp(0.0, 90.0);

        if !self.reflectance_color.is_zero() {
            if sigma == 0.0 {
                let lambertian = Lambertian::new(self.reflectance_color);
                bsdf.add(Bxdf::Lambertian(lambertian));
            } else {
                let oren_nayar = OrenNayar::new(self.reflectance_color, self.roughness);
                bsdf.add(Bxdf::OrenNayar(oren_nayar));
            }
        }
        si.bsdf = Some(bsdf);
    }

    fn get_albedo(&self) -> Vector3<f64> {
        self.reflectance_color
    }
}
