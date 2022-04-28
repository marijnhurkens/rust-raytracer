use nalgebra::Vector3;
use bsdf::{BSDF, BXDF};
use bsdf::lambertian::Lambertian;
use materials::MaterialTrait;
use surface_interaction::SurfaceInteraction;

#[derive(Debug)]
pub struct MatteMaterial {
    pub reflectance_color: Vector3<f64>,
    pub roughness: f64,
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
        let mut bsdf = BSDF::new(*si, None);

        let lambertian = Lambertian::new(self.reflectance_color);

        bsdf.add(BXDF::Lambertian(lambertian));

        si.bsdf = Some(bsdf);
    }
}
