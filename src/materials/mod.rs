use nalgebra::Vector3;
use bsdf::lambertian::Lambertian;
use bsdf::{BSDF, BXDF};
use surface_interaction::SurfaceInteraction;

#[derive(Debug)]
pub enum Material {
    MatteMaterial(MatteMaterial),
}

impl MaterialTrait for Material {
    fn compute_scattering_functions(&self, si: &mut SurfaceInteraction) {
        match self {
            Material::MatteMaterial(x) => x.compute_scattering_functions(si)
        }
    }
}


pub trait MaterialTrait {
    fn compute_scattering_functions(&self, si: &mut SurfaceInteraction);
}




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
        let mut bsdf = BSDF::new(si.clone(), None);

        let lambertian = Lambertian::new(self.reflectance_color);

        bsdf.add(BXDF::Lambertian(lambertian));

        si.bsdf = Some(bsdf);
    }
}
