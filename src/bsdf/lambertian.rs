use nalgebra::{Point3, Vector3};

use crate::bsdf::helpers::{abs_cos_theta, get_cosine_weighted_in_hemisphere, same_hemisphere};
use crate::bsdf::{BXDFtrait, BXDFTYPES};
use crate::renderer::debug_write_pixel_f64;

#[derive(Debug, Clone, Copy)]
pub struct Lambertian {
    reflectance_color: Vector3<f64>,
}

impl Lambertian {
    pub fn new(reflectance_color: Vector3<f64>) -> Self {
        Lambertian { reflectance_color }
    }
}

impl BXDFtrait for Lambertian {
    fn get_type_flags(&self) -> BXDFTYPES {
        BXDFTYPES::REFLECTION | BXDFTYPES::DIFFUSE
    }

    fn f(&self, _wo: Vector3<f64>, _wi: Vector3<f64>) -> Vector3<f64> {
        self.reflectance_color * std::f64::consts::FRAC_1_PI
    }

    fn pdf(&self, wo: Vector3<f64>, wi: Vector3<f64>) -> f64 {
        if same_hemisphere(wo, wi) {
            abs_cos_theta(wi) * std::f64::consts::FRAC_1_PI
        } else {
            0.0
        }
    }

    fn sample_f(&self, _point: Point3<f64>, wo: Vector3<f64>) -> (Vector3<f64>, f64, Vector3<f64>) {
        let mut wi = get_cosine_weighted_in_hemisphere();
        if wo.z < 0.0 {
            wi.z = -wi.z;
        }

        (wi, self.pdf(wo, wi), self.f(wo, wi))
    }
}
