use nalgebra::Vector3;
use rand::{thread_rng, Rng};
use rand::prelude::SliceRandom;

use bsdf::BXDFTYPES;
use materials::MaterialTrait;
use objects::Objectable;
use renderer::{check_intersect_scene, check_light_visible, Ray, Settings};
use scene::Scene;
use surface_interaction::SurfaceInteraction;

pub fn trace(
    settings: &Settings,
    ray: Ray,
    scene: &Scene,
) -> Option<(Vector3<f64>, Vector3<f64>)> {
    let mut l = Vector3::new(0.0, 0.0, 0.0);
    let mut contribution = 1.0;
    let mut specular_bounce = false;

    for bounce in 0..settings.depth_limit {
        let intersect = check_intersect_scene(ray, scene);

        let (mut surface_interaction, object) = match intersect {
            Some(intersection) => intersection,
            None => {
                // terminate
                return Some((scene.bg_color, Vector3::new(0.0, 0.0, 0.0)));
            }
        };

        if bounce == 0 || specular_bounce {
            //l += contribution * intersection.le();
        }

        for material in object.get_materials() {
            material.compute_scattering_functions(&mut surface_interaction);
        }

        l += contribution * uniform_sample_light(scene, surface_interaction)
    }

    Some((l, Vector3::zeros()))
}

fn uniform_sample_light(scene: &Scene, surface_interaction: SurfaceInteraction) -> Vector3<f64> {
    let mut rng = thread_rng();

    let light = scene.lights.choose(&mut rng).unwrap();

    if !check_light_visible(surface_interaction.point, scene, light) {
        return Vector3::zeros();
    }

    let wi = light.position - surface_interaction.point;

    surface_interaction
        .bsdf
        .unwrap()
        .f(surface_interaction.wo, wi, BXDFTYPES::ALL)
}
