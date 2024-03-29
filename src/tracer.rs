use std::borrow::BorrowMut;

use nalgebra::{Point2, Point3, SimdPartialOrd, Vector3};
use num_traits::identities::Zero;
use rand::prelude::SliceRandom;
use rand::{thread_rng, Rng};

use crate::bsdf::{BsdfSampleResult, BXDFTYPES};
use crate::helpers::power_heuristic;
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
    let mut rng = thread_rng();
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

        if bounce == 0 {
            normal = surface_interaction.shading_normal;
            albedo = object.get_materials()[0].get_albedo()
        }

        for material in object.get_materials() {
            material.compute_scattering_functions(&mut surface_interaction);
        }

        let mut light_irradiance = uniform_sample_light(scene, &surface_interaction, sampler);

        // clamp indirect light?
        // if bounce > 0 {
        //     light_irradiance = light_irradiance.simd_clamp(Vector3::zeros(), Vector3::repeat(10.0));
        // }

        l += contribution.component_mul(&light_irradiance);

        let wo = -ray.direction;
        let bsdf_sample = surface_interaction
            .bsdf
            .as_ref()
            .unwrap()
            .sample_f(wo, BXDFTYPES::ALL);

        if bsdf_sample.pdf == 0.0 || bsdf_sample.f.is_zero() {
            break;
        }

        contribution = contribution.component_mul(
            &((bsdf_sample.f
                * bsdf_sample
                    .wi
                    .dot(&surface_interaction.shading_normal)
                    .abs())
                / bsdf_sample.pdf),
        );

        // if contribution.max() > 300.0 {
        //     dbg!(bsdf_sample.f, bsdf_sample.pdf);
        //     panic!();
        // }

        specular_bounce = bsdf_sample.sampled_flags.contains(BXDFTYPES::SPECULAR);

        ray = Ray {
            point: surface_interaction.point,
            direction: bsdf_sample.wi,
        };

        // russian roulette termination
        if bounce > 3 {
            let q = (1.0 - contribution.max()).max(0.05);
            if rng.gen::<f64>() < q {
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
    let mut rng = thread_rng();
    let bsdf_flags = BXDFTYPES::ALL & !BXDFTYPES::SPECULAR;

    let mut direct_irradiance = Vector3::zeros();

    let light = scene.lights.choose(&mut rng).unwrap();

    // Sample a random point on the light and calculate the irradiance at our intersection point.
    let u_light = sampler.get_3d();
    // todo: fix, black spots when pulling samples here
    //let u_light = vec!(1.0,1.0);
    let mut irradiance_sample = light.sample_irradiance(surface_interaction, u_light);

    // First we calculate the BSDF value for our light sample
    if irradiance_sample.pdf > 0.0 && !irradiance_sample.irradiance.is_zero() {
        let mut f = if let Some(bsdf) = surface_interaction.bsdf.as_ref() {
            bsdf.f(surface_interaction.wo, irradiance_sample.wi, bsdf_flags)
        } else {
            Vector3::zeros()
        };

        f *= irradiance_sample
            .wi
            .dot(&surface_interaction.shading_normal)
            .abs();

        if !f.is_zero() {
            if !check_light_visible(surface_interaction, scene, &irradiance_sample) {
                irradiance_sample.irradiance = Vector3::zeros();
            }

            if !irradiance_sample.irradiance.is_zero() {
                if light.is_delta() {
                    direct_irradiance +=
                        f.component_mul(&irradiance_sample.irradiance) / irradiance_sample.pdf;
                } else {
                    let scattering_pdf = if let Some(bsdf) = surface_interaction.bsdf.as_ref() {
                        bsdf.pdf(surface_interaction.wo, irradiance_sample.wi, bsdf_flags)
                    } else {
                        0.0
                    };

                    let weight = power_heuristic(1, irradiance_sample.pdf, 1, scattering_pdf);
                    direct_irradiance += f.component_mul(&irradiance_sample.irradiance) * weight
                        / irradiance_sample.pdf;
                }
            }
        }
    }

    if !light.is_delta() {
        let bsdf_sample = if let Some(bsdf) = surface_interaction.bsdf.as_ref() {
            bsdf.sample_f(surface_interaction.wo, bsdf_flags)
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

        if !f.is_zero() && bsdf_sample.pdf > 0.0 {
            let interaction = Interaction {
                point: surface_interaction.point,
                normal: surface_interaction.shading_normal,
            };
            let light_pdf = light.pdf_incidence(&interaction, bsdf_sample.wi);
            if light_pdf == 0.0 {
                return direct_irradiance;
            }

            let weight = power_heuristic(1, bsdf_sample.pdf, 1, light_pdf);

            let ray = Ray {
                point: surface_interaction.point + (bsdf_sample.wi * 1.0e-9),
                direction: bsdf_sample.wi,
            };

            let mut light_irradiance = Vector3::zeros();

            if let Some((object_interaction, object)) = check_intersect_scene(ray, scene) {
                if let Some(found_light_arc) = object.get_light() {
                    if std::ptr::eq(light.as_ref(), found_light_arc.as_ref()) {
                        if let Light::Area(light) = light.as_ref() {
                            // we've hit OUR area light
                            let interaction = Interaction {
                                point: object_interaction.point,
                                normal: object_interaction.shading_normal,
                            };
                            light_irradiance =
                                light.irradiance_at_point(&interaction, -bsdf_sample.wi);
                        }
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

            direct_irradiance += f.component_mul(&(light_irradiance * weight)) / bsdf_sample.pdf;
        }
    }

    direct_irradiance
}
