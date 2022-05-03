use nalgebra::Vector3;
use num_traits::identities::Zero;
use rand::prelude::SliceRandom;
use rand::{thread_rng, Rng};

use bsdf::BXDFTYPES;
use lights::LightTrait;
use materials::MaterialTrait;
use objects::Objectable;
use renderer::{check_intersect_scene, check_light_visible, debug_write_pixel, debug_write_pixel_f64, Ray, Settings};
use scene::Scene;
use surface_interaction::SurfaceInteraction;

pub fn trace(
    settings: &Settings,
    starting_ray: Ray,
    scene: &Scene,
) -> Option<(Vector3<f64>, Vector3<f64>)> {
    let mut rng = thread_rng();
    let mut l = Vector3::new(0.0, 0.0, 0.0);
    let mut contribution = Vector3::new(1.0, 1.0, 1.0);
    let mut specular_bounce = false;
    let mut ray = starting_ray;
    let mut normal = Vector3::zeros();

    for bounce in 0..settings.depth_limit {
        let intersect = check_intersect_scene(ray, scene);

        // Check for an intersection
        let (mut surface_interaction, object) = match intersect {
            Some(intersection) => intersection,
            None => {
                break;
            }
        };

        if bounce == 0 {
            normal = surface_interaction.geometry_normal;
        }

        if bounce == 0 || specular_bounce {
            //l += contribution * intersection.le();
        }

        for material in object.get_materials() {
            material.compute_scattering_functions(&mut surface_interaction);
        }

        let light_irradiance = uniform_sample_light(scene, &surface_interaction);

        l += contribution.component_mul(&light_irradiance);

        let wo = -ray.direction;
        let (wi, pdf, f) = surface_interaction
            .bsdf
            .as_ref()
            .unwrap()
            .sample_f(wo, BXDFTYPES::ALL);

        //debug_write_pixel(wi);
        // todo: fix
        if pdf == 0.0 || f.is_zero() {
            break;
        }

        contribution = contribution
            .component_mul(&(f * wi.dot(&surface_interaction.shading_normal).abs() / pdf));

        ray = Ray {
            point: surface_interaction.point,
            direction: wi,
        };

        // russian roulette termination
        if bounce > 3 {
            let q = (1.0 - contribution.y).max(0.05);
            if rng.gen::<f64>() < q {
                break;
            }

            contribution /= 1.0 - q;
        }
    }

    Some((l, normal))
}

fn uniform_sample_light(scene: &Scene, surface_interaction: &SurfaceInteraction) -> Vector3<f64> {
    let mut rng = thread_rng();
    let bsdf_flags = BXDFTYPES::ALL & !BXDFTYPES::SPECULAR;

    let mut direct_irradiance = Vector3::zeros();

    let light = scene.lights.choose(&mut rng).unwrap();
    let mut irradiance = light.sample_irradiance(surface_interaction);
    let wi = (light.get_position() - surface_interaction.point).normalize();
    let pdf = 1.0;

    if pdf > 0.0 || !irradiance.is_zero() {
        let f = surface_interaction.bsdf.as_ref().unwrap().f(
            surface_interaction.wo,
            wi,
            bsdf_flags,
        ) * wi.dot(&surface_interaction.shading_normal).abs();

        if !f.is_zero() {
            if !check_light_visible(surface_interaction.point, scene, light) {
                irradiance = Vector3::zeros();
            }


            if !irradiance.is_zero() {
                direct_irradiance += f.component_mul(&irradiance) / pdf;
            }
        }
    }

    direct_irradiance
}
