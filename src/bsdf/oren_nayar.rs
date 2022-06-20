use std::f64::consts::FRAC_1_PI;
use nalgebra::{Point3, Vector3};

use crate::bsdf::helpers::{abs_cos_theta, cos_phi, get_cosine_weighted_in_hemisphere, same_hemisphere, sin_phi, sin_theta};
use crate::bsdf::{BXDFtrait, BXDFTYPES};

#[derive(Debug, Clone, Copy)]
pub struct OrenNayar {
    reflectance_color: Vector3<f64>,
    a: f64,
    b: f64,
}

impl OrenNayar {
    pub fn new(reflectance_color: Vector3<f64>, roughness: f64) -> Self {
        let sigma2 = roughness * roughness;
        let a = 1.0 - (sigma2 / (2.0 * (sigma2 + 0.33)));
        let b = 0.45 * sigma2 / (sigma2 + 0.09);

        OrenNayar {
            reflectance_color,
            a,
            b,
        }
    }
}

impl BXDFtrait for OrenNayar {
    fn get_type_flags(&self) -> BXDFTYPES {
        BXDFTYPES::REFLECTION | BXDFTYPES::DIFFUSE
    }

    fn f(&self, wo: Vector3<f64>, wi: Vector3<f64>) -> Vector3<f64> {
        let sin_theta_i = sin_theta(wi);
        let sin_theta_o = sin_theta(wo);

        let mut max_cos = 0.0;
        if sin_theta_i > 1e-4 && sin_theta_o > 1e-4 {
            let sin_phi_i = sin_phi(wi);
            let cos_phi_i = cos_phi(wi);
            let sin_phi_o = sin_phi(wo);
            let cos_phi_o = cos_phi(wo);
            let d_cos = cos_phi_i * cos_phi_o + sin_phi_i * sin_phi_o;
            max_cos = d_cos.max(0.0);
        }

        let sin_alpha;
        let tan_beta;

        if abs_cos_theta(wi) > abs_cos_theta(wo) {
            sin_alpha = sin_theta_o;
            tan_beta = sin_theta_i / abs_cos_theta(wi);
        } else {
            sin_alpha = sin_theta_i;
            tan_beta = sin_theta_o / abs_cos_theta(wo);
        }

        self.reflectance_color
            * FRAC_1_PI
            * (self.a + self.b * max_cos * sin_alpha * tan_beta)
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
