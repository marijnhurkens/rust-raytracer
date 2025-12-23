use std::f64::consts::{FRAC_1_PI, FRAC_2_PI, PI};
use std::path::PathBuf;

use ggez::graphics::Image;
use image::Pixel;
use image::{ImageBuffer, Rgb, RgbImage};
use nalgebra::{Matrix3, Matrix4, Point2, Point3, Transform, Vector3};

use crate::helpers::{get_random_in_unit_sphere, spherical_phi, spherical_theta};
use crate::lights::{LightEmittingPdf, LightEmittingSample, LightIrradianceSample, LightTrait};
use crate::renderer::Ray;
use crate::surface_interaction::{Interaction, SurfaceInteraction};
use crate::textures::mip_map::MipMap;
use crate::film::srgb_to_xyz;


#[derive(Debug)]
pub struct InfiniteAreaLight {
    mip_map: MipMap,
    light_to_world: Matrix4<f64>,
    world_to_light: Matrix4<f64>,
    world_center: Point3<f64>,
    world_radius: f64,
}

impl LightTrait for InfiniteAreaLight {
    fn is_delta(&self) -> bool {
        false
    }

    fn emitting(&self, interaction: &SurfaceInteraction, wi: Vector3<f64>) -> Vector3<f64> {
        // let w_light = self.world_to_light.transform_vector(&w).normalize();
        //
        // let p = Point2::new(spherical_phi(w_light) * FRAC_2_PI,  spherical_theta(w_light) * FRAC_1_PI);

        let ray = Ray {
            point: interaction.point,
            direction: wi.normalize(),
        };

        self.environment_emitting(ray)

    }

    fn sample_irradiance(
        &self,
        interaction: &SurfaceInteraction,
        sample: Vec<f64>,
    ) -> LightIrradianceSample {
        let theta = sample[1] * PI;
        let phi = sample[0] * 2.0 * PI;
        let cos_theta = theta.cos();
        let sin_theta = theta.sin();
        let cos_phi = phi.cos();
        let sin_phi = phi.sin();

        let wi = self.light_to_world.transform_vector(&Vector3::new(
            sin_theta * sin_phi,
            cos_theta,
            sin_theta * cos_phi,
        ));

        let ray = Ray {
            point: interaction.point,
            direction: wi.normalize(),
        };

        let pdf = if sin_theta != 0.0 {
            self.environment_emitting(ray).max() / (2.0 * PI * PI * sin_theta)
        } else {
            0.0
        };

        let point_outside = interaction.point + wi * (2.0 * self.world_radius);

        LightIrradianceSample {
            point: point_outside,
            wi,
            pdf,
            irradiance: self.environment_emitting(ray),
        }
    }

    fn sample_emitting(&self) -> LightEmittingSample {
        todo!()
    }

    fn pdf_incidence(&self, interaction: &Interaction, wi: Vector3<f64>) -> f64 {
        1.0
    }

    fn pdf_emitting(&self, ray: Ray, light_normal: Vector3<f64>) -> LightEmittingPdf {
        todo!()
    }

    fn environment_emitting(&self, ray: Ray) -> Vector3<f64> {
        let w = self.world_to_light.transform_vector(&ray.direction);
        let point = Point2::new(
            1.0 - spherical_phi(w) * 1.0 / (2.0 * PI),
            spherical_theta(w) * FRAC_1_PI,
        );

        let lookup = self.mip_map.lookup(point, 0.5);

        Vector3::new(lookup[0], lookup[1], lookup[2])
    }

    fn power(&self) -> Vector3<f64> {
        let lookup = self.mip_map.lookup(Point2::new(0.5, 0.5), 0.5);
        Vector3::new(lookup[0], lookup[1], lookup[2]) * PI * self.world_radius * self.world_radius
    }
}

impl InfiniteAreaLight {
    pub fn new(intensity: &Vector3<f64>, image: RgbImage, light_to_world: Matrix4<f64>) -> Self {
        let mut buffer = ImageBuffer::new(image.width(), image.height());
        for (x, y, pixel) in image.enumerate_pixels() {
            let pixel_xyz = srgb_to_xyz(
                Vector3::new(pixel[0] as f64 / 255.0, pixel[1] as f64/ 255.0, pixel[2] as f64/ 255.0)
            );
            let adjusted_pixel = Rgb([
                ((pixel_xyz.x * intensity.x) * 255.0).min(255.0) as u8,
                ((pixel_xyz.y * intensity.y) * 255.0).min(255.0) as u8,
                ((pixel_xyz.z * intensity.z) * 255.0).min(255.0) as u8,
            ]);

            buffer.put_pixel(x, y, adjusted_pixel)
        }

        let mip_map = MipMap::new(buffer);

        InfiniteAreaLight {
            mip_map,
            light_to_world,
            world_to_light: light_to_world.try_inverse().unwrap(),
            world_center: Point3::origin(),
            world_radius: 1e20,
        }
    }
}
