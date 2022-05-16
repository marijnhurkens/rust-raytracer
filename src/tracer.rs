use nalgebra::Vector3;
use num_traits::identities::Zero;
use rand::prelude::SliceRandom;
use rand::{thread_rng, Rng};

use bsdf::BXDFTYPES;
use helpers::power_heuristic;
use lights::{Light, LightTrait};
use materials::MaterialTrait;
use objects::ObjectTrait;
use renderer::{
    check_intersect_scene, check_intersect_scene_simple, check_light_visible, debug_write_pixel,
    debug_write_pixel_f64, Ray, Settings,
};
use scene::Scene;
use surface_interaction::{Interaction, SurfaceInteraction};

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
            normal = surface_interaction.shading_normal;
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
    let mut irradiance_sample = light.sample_irradiance(surface_interaction);

    if irradiance_sample.pdf > 0.0 || !irradiance_sample.irradiance.is_zero() {
        let mut f = if let Some(bsdf) = surface_interaction.bsdf.as_ref() {
            bsdf.f(surface_interaction.wo, irradiance_sample.wi, bsdf_flags)
        } else {
            Vector3::zeros()
        };

        f *= irradiance_sample
            .wi
            .dot(&surface_interaction.shading_normal)
            .abs();

        let scattering_pdf = if let Some(bsdf) = surface_interaction.bsdf.as_ref() {
            bsdf.pdf(surface_interaction.wo, irradiance_sample.wi, bsdf_flags)
        } else {
            0.0
        };

        if !f.is_zero() {
            if !check_light_visible(surface_interaction, scene, &irradiance_sample) {
                irradiance_sample.irradiance = Vector3::zeros();
            }

            if !irradiance_sample.irradiance.is_zero() {
                if light.is_delta() {
                    direct_irradiance +=
                        f.component_mul(&irradiance_sample.irradiance) / irradiance_sample.pdf;
                } else {
                    let weight = power_heuristic(1, irradiance_sample.pdf, 1, scattering_pdf);
                    let light_irradiance = irradiance_sample.irradiance * weight;
                    direct_irradiance += f.component_mul(&light_irradiance) / irradiance_sample.pdf;
                }
            }
        }
    }

    if !light.is_delta() {
        let (wi, scattering_pdf, f) = if let Some(bsdf) = surface_interaction.bsdf.as_ref() {
            bsdf.sample_f(surface_interaction.wo, bsdf_flags)
        } else {
            (Vector3::zeros(), 0.0, Vector3::zeros())
        };

        let f = f * wi.dot(&surface_interaction.shading_normal).abs();

        if !f.is_zero() && scattering_pdf > 0.0 {
            let interaction = Interaction {
                point: surface_interaction.point,
                normal: surface_interaction.shading_normal,
            };
            let light_pdf = light.pdf_incidence(&interaction, wi);
            if light_pdf == 0.0 {
                return direct_irradiance;
            }

            let weight = power_heuristic(1, scattering_pdf, 1, light_pdf);

            let ray = Ray {
                point: surface_interaction.point + (wi * 1.0e-9),
                direction: wi,
            };

            let mut light_irradiance = Vector3::zeros();

            if let Some((object_interaction, object)) = check_intersect_scene(ray, &scene) {
                let light_option = object.get_light();
                if let Some(light) = light_option {
                    if let Light::Area(light) = light.as_ref() {
                        // we've hit an area light
                        let interaction = Interaction {
                            point: object_interaction.point,
                            normal: object_interaction.shading_normal,
                        };
                        light_irradiance = light.irradiance_at_point(&interaction, -wi);
                    }
                }
            } else {
                // no hit, add emitting light if infinite area light
                // let interaction = Interaction {
                //     point: surface_interaction.point,
                //     normal: surface_interaction.shading_normal,
                // };
                // light_irradiance = light.emitting(&interaction, -wi)
            }

            direct_irradiance += f.component_mul(&(light_irradiance * weight)) / scattering_pdf;
        }
    }

    direct_irradiance
}
