use materials::matte::MatteMaterial;
use materials::plastic::PlasticMaterial;
use surface_interaction::SurfaceInteraction;

pub mod matte;
pub mod plastic;

#[derive(Debug, Clone, PartialEq)]
pub enum Material {
    MatteMaterial(MatteMaterial),
    PlasticMaterial(PlasticMaterial)
}

pub trait MaterialTrait {
    fn compute_scattering_functions(&self, si: &mut SurfaceInteraction);
}

impl MaterialTrait for Material {
    fn compute_scattering_functions(&self, si: &mut SurfaceInteraction) {
        match self {
            Material::MatteMaterial(x) => x.compute_scattering_functions(si),
            Material::PlasticMaterial(x) => x.compute_scattering_functions(si),
        }
    }
}
