use nalgebra::{Point2, Point3, SimdPartialOrd, Vector3};
use num_traits::identities::Zero;
use rand::prelude::{IteratorRandom, SliceRandom};
use rand::{rng, Rng};
use std::borrow::BorrowMut;
use std::sync::Arc;

use crate::bsdf::{BsdfSampleResult, BXDFTYPES};
use crate::helpers::{offset_ray_origin, power_heuristic};
use crate::lights::area::AreaLight;
use crate::lights::{Light, LightTrait};
use crate::materials::MaterialTrait;
use crate::objects::plane::Plane;
use crate::objects::ObjectTrait;
use crate::renderer::{
    check_intersect_scene, check_intersect_scene_simple, check_light_visible, debug_write_pixel,
    debug_write_pixel_f64, debug_write_pixel_f64_on_bounce, debug_write_pixel_on_bounce, Ray,
    SampleResult, Settings, CURRENT_BOUNCE,
};
use crate::scene::Scene;
use crate::surface_interaction::{Interaction, SurfaceInteraction};
use crate::{Object, SobolSampler};

pub fn trace(
    starting_ray: Ray,
    point_film: Point2<f64>,
    settings: &Settings,
    scene: &Scene,
    sampler: &mut SobolSampler,
) -> SampleResult {
    let mut rng = rng();
    let mut l = Vector3::new(0.0, 0.0, 0.0);
    let mut contribution = Vector3::new(1.0, 1.0, 1.0);
    let mut specular_bounce = false;
    let mut ray = starting_ray;
    let mut normal = Vector3::zeros();
    let mut albedo = Vector3::zeros();

    for bounce in 0..settings.depth_limit {
        CURRENT_BOUNCE.with(|current_bounce| *current_bounce.borrow_mut() = bounce);

        let intersect = check_intersect_scene(ray, scene);

        if bounce == 0 || specular_bounce {
            // If we hit a light source on the first bounce or after a specular bounce, add its contribution
            if let Some((interaction, object)) = intersect {
                if let Some(light) = object.get_light() {
                    l += contribution.component_mul(&light.emitting(&interaction, -ray.direction));
                }
            } else {
                for light in &scene.lights {
                    l += contribution.component_mul(&light.environment_emitting(ray));
                }
            }
        }

        // Check for an intersection
        let (mut surface_interaction, object) = match intersect {
            Some(intersection) => intersection,
            None => {
                break;
            }
        };

        // if bounce == 0 {
        //     normal = surface_interaction.shading_normal;
        //     albedo = object.get_materials()[0].get_albedo()
        // }

        for material in object.get_materials() {
            material.compute_scattering_functions(&mut surface_interaction);
        }

        let mut light_irradiance = uniform_sample_light(scene, &surface_interaction, sampler);
        l += contribution.component_mul(&light_irradiance);

        let bsdf_sample = surface_interaction.bsdf.as_ref().unwrap().sample_f(
            surface_interaction.wo,
            BXDFTYPES::ALL,
            sampler.get_2d_point(),
        );

        if bsdf_sample.pdf == 0.0 || bsdf_sample.f.is_zero() {
            break;
        }

        let contribution_before = contribution.clone();
        contribution = contribution.component_mul(
            &((bsdf_sample.f
                * bsdf_sample
                    .wi
                    .dot(&surface_interaction.shading_normal)
                    .abs())
                / bsdf_sample.pdf),
        );
        // if contribution.x > 100.0 || contribution.y > 100.0 || contribution.z > 100.0
        // {
        //     dbg!(contribution_before, contribution, bsdf_sample.f, bsdf_sample
        //             .wi
        //             .dot(&surface_interaction.shading_normal)
        //             .abs(), bsdf_sample.pdf);
        // debug_write_pixel(Vector3::new(1.0,0.0,0.0));
        // }

        specular_bounce = bsdf_sample.sampled_flags.contains(BXDFTYPES::SPECULAR);

        ray = Ray {
            point: offset_ray_origin(
                surface_interaction.point,
                surface_interaction.geometry_normal,
                bsdf_sample.wi,
            ),
            direction: bsdf_sample.wi,
        };

        if settings.clamp > 0.0 {
            l.x = l.x.min(settings.clamp);
            l.y = l.y.min(settings.clamp);
            l.z = l.z.min(settings.clamp);
        }

        // russian roulette termination
        if bounce > 3 {
            let q = (1.0 - contribution.max()).max(0.05);
            if rng.random::<f64>() < q {
                break;
            }

            contribution /= 1.0 - q;
        }
    }

    SampleResult {
        radiance: l,
        p_film: point_film,
        normal,
        albedo,
    }
}

fn uniform_sample_light(
    scene: &Scene,
    surface_interaction: &SurfaceInteraction,
    sampler: &mut SobolSampler,
) -> Vector3<f64> {
    let mut rng = rng();

    let light_count = scene.lights.len();
    let light_num = (sampler.get_1d() * light_count as f64).min(light_count as f64 - 1.0);
    let light = &scene.lights[light_num as usize];

    let light_pdf = 1.0 / light_count as f64;

    estimate_direct(scene, surface_interaction, sampler, light) / light_pdf
}

fn estimate_direct(
    scene: &Scene,
    surface_interaction: &SurfaceInteraction,
    sampler: &mut SobolSampler,
    light: &Arc<Light>,
) -> Vector3<f64> {
    let bsdf_flags = BXDFTYPES::ALL & !BXDFTYPES::SPECULAR;
    let mut direct_irradiance = Vector3::zeros();

    // Sample light source with multiple importance sampling
    let u_light = sampler.get_3d();
    // todo: fix, black spots when pulling samples here
    //let u_light = vec!(1.0,1.0);
    let mut irradiance_sample = light.sample_irradiance(surface_interaction, u_light);
    let light_pdf = irradiance_sample.pdf;
    let mut scattering_pdf = 0.0;

    if irradiance_sample.pdf > 1e-6 && !irradiance_sample.irradiance.is_zero() {
        // First we calculate the BSDF value for our light sample
        let mut f = if let Some(bsdf) = surface_interaction.bsdf.as_ref() {
            bsdf.f(surface_interaction.wo, irradiance_sample.wi, bsdf_flags)
        } else {
            Vector3::zeros()
        };

        f *= irradiance_sample
            .wi
            .dot(&surface_interaction.shading_normal)
            .abs();
        scattering_pdf = surface_interaction.bsdf.unwrap().pdf(surface_interaction.wo, irradiance_sample.wi, bsdf_flags);

        if !f.is_zero() {
            if !check_light_visible(surface_interaction, scene, &irradiance_sample) {
                irradiance_sample.irradiance = Vector3::zeros();
            }

            if light.is_delta() {
                direct_irradiance +=
                    f.component_mul(&irradiance_sample.irradiance) / light_pdf;
            } else {
                let weight = power_heuristic(1, light_pdf, 1, scattering_pdf);

                direct_irradiance +=
                    f.component_mul(&irradiance_sample.irradiance) * weight / light_pdf;
            }
        }
    }

    // Sample BSDF with multiple importance sampling
    if !light.is_delta() {
        let mut sampled_specular = false;

        let bsdf_sample = if let Some(bsdf) = surface_interaction.bsdf.as_ref() {
            bsdf.sample_f(surface_interaction.wo, bsdf_flags, sampler.get_2d_point())
        } else {
            BsdfSampleResult {
                wi: Vector3::zeros(),
                pdf: 0.0,
                f: Vector3::zeros(),
                sampled_flags: BXDFTYPES::NONE,
            }
        };

        let f = bsdf_sample.f
            * bsdf_sample
                .wi
                .dot(&surface_interaction.shading_normal)
                .abs();
        scattering_pdf = bsdf_sample.pdf;
        sampled_specular = bsdf_sample.sampled_flags.contains(BXDFTYPES::SPECULAR);

        if !f.is_zero() && scattering_pdf > 1e-6 {
            // Account for light contributions along sampled direction _wi_
            let interaction = Interaction {
                point: surface_interaction.point,
                normal: surface_interaction.shading_normal,
            };
            let mut weight = 1.0;
            if !sampled_specular {
                let light_pdf = light.pdf_incidence(&interaction, bsdf_sample.wi);
                if light_pdf < 1e-6 {
                    return direct_irradiance;
                }
                weight = power_heuristic(1, scattering_pdf, 1, light_pdf);
            }

            let ray = Ray {
                point: offset_ray_origin(
                    surface_interaction.point,
                    surface_interaction.geometry_normal,
                    bsdf_sample.wi,
                ),
                direction: bsdf_sample.wi,
            };

            let mut light_irradiance = Vector3::zeros();

            if let Some((object_interaction, object)) = check_intersect_scene(ray, scene) {
                if let Some(found_light_arc) = object.get_light() {
                    if std::ptr::eq(light.as_ref(), found_light_arc.as_ref()) {
                        if let Light::Area(light) = light.as_ref() {
                            // // we've hit OUR area light
                            // let interaction = Interaction {
                            //     point: object_interaction.point,
                            //     normal: object_interaction.shading_normal,
                            // };
                            light_irradiance =
                                light.emitting(&object_interaction, -bsdf_sample.wi);
                        }
                    }
                }
            } else {
                // // no hit, add emitting light if infinite area light
                // let interaction = Interaction {
                //     point: surface_interaction.point,
                //     normal: surface_interaction.shading_normal,
                // };

                light_irradiance = light.emitting(&surface_interaction, ray.direction)
            }

            if !light_irradiance.is_zero() {
                direct_irradiance += f.component_mul(&(light_irradiance * weight)) / bsdf_sample.pdf;
            }
        }
    }

    direct_irradiance
}
