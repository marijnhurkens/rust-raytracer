use nalgebra::Vector3;
use crate::materials::matte::MatteMaterial;
use crate::materials::plastic::PlasticMaterial;
use crate::surface_interaction::SurfaceInteraction;

pub mod matte;
pub mod plastic;

#[derive(Debug, Clone, PartialEq)]
pub enum Material {
    MatteMaterial(MatteMaterial),
    PlasticMaterial(PlasticMaterial),
}

pub trait MaterialTrait {
    fn compute_scattering_functions(&self, si: &mut SurfaceInteraction);
    fn get_albedo(&self) -> Vector3<f64>;
}

impl MaterialTrait for Material {
    fn compute_scattering_functions(&self, si: &mut SurfaceInteraction) {
        match self {
            Material::MatteMaterial(x) => x.compute_scattering_functions(si),
            Material::PlasticMaterial(x) => x.compute_scattering_functions(si),
        }
    }

    fn get_albedo(&self) -> Vector3<f64> {
        match self {
            Material::MatteMaterial(x) => x.get_albedo(),
            Material::PlasticMaterial(x) => x.get_albedo(),
        }
    }
}
