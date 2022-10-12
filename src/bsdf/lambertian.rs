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
}
