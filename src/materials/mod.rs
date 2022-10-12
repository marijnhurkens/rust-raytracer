use nalgebra::Vector3;
use crate::materials::glass::GlassMaterial;

use crate::materials::matte::MatteMaterial;
use crate::materials::mirror::MirrorMaterial;
use crate::materials::plastic::PlasticMaterial;
use crate::surface_interaction::SurfaceInteraction;

pub mod matte;
pub mod mirror;
pub mod plastic;
pub mod glass;

#[derive(Debug, Clone, PartialEq)]
pub enum Material {
    Matte(MatteMaterial),
    Plastic(PlasticMaterial),
    Mirror(MirrorMaterial),
    Glass(GlassMaterial),
}

pub trait MaterialTrait {
    fn compute_scattering_functions(&self, si: &mut SurfaceInteraction);
    fn get_albedo(&self) -> Vector3<f64>;
}

impl MaterialTrait for Material {
    fn compute_scattering_functions(&self, si: &mut SurfaceInteraction) {
        match self {
            Material::Matte(x) => x.compute_scattering_functions(si),
            Material::Plastic(x) => x.compute_scattering_functions(si),
            Material::Mirror(x) => x.compute_scattering_functions(si),
            Material::Glass(x) => x.compute_scattering_functions(si),
        }
    }

    fn get_albedo(&self) -> Vector3<f64> {
        match self {
            Material::Matte(x) => x.get_albedo(),
            Material::Plastic(x) => x.get_albedo(),
            Material::Mirror(x) => x.get_albedo(),
            Material::Glass(x) => x.get_albedo(),
        }
    }
}
