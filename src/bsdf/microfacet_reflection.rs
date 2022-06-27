use nalgebra::{Point3, Vector3};
use num_traits::Zero;
use crate::bsdf::helpers::{get_cosine_weighted_in_hemisphere, same_hemisphere};

use super::{BXDFtrait, BXDFTYPES};
use super::helpers::abs_cos_theta;
use super::helpers::fresnel::{DielectricFresnel, Fresnel};
use super::helpers::microfacet_distribution::{MicrofacetDistribution, TrowbridgeReitzDistribution};


#[derive(Debug, Copy, Clone)]
pub struct MicrofacetReflection {
    reflectance_color: Vector3<f64>,
    distribution: TrowbridgeReitzDistribution,
    fresnel: DielectricFresnel,
}

impl MicrofacetReflection {
    pub fn new(
        reflectance_color: Vector3<f64>,
        distribution: TrowbridgeReitzDistribution,
        fresnel: DielectricFresnel,
    ) -> Self {
        MicrofacetReflection {
            reflectance_color,
            distribution,
            fresnel,
        }
    }
}

impl BXDFtrait for MicrofacetReflection {
    fn get_type_flags(&self) -> BXDFTYPES {
        BXDFTYPES::REFLECTION | BXDFTYPES::GLOSSY
    }

    fn f(&self, wo: Vector3<f64>, wi: Vector3<f64>) -> Vector3<f64> {
        let cos_theta_o = abs_cos_theta(wo);
        let cos_theta_i = abs_cos_theta(wi);
        if cos_theta_o == 0.0 || cos_theta_i == 0.0 {
            return Vector3::zeros();
        }

        let wh = wi + wo;

        if wh.is_zero() { return Vector3::zeros(); }

        let wh = wh.normalize();
        let f = self.fresnel.evaluate(wi.dot(&wh));
        self.reflectance_color * self.distribution.d(wh) * self.distribution.g(wo, wi) * f /
            (4.0 * cos_theta_i * cos_theta_o)
    }

    fn pdf(&self, wo: Vector3<f64>, wi: Vector3<f64>) -> f64 {
        // todo: check if correct, copied from lambertian
        if same_hemisphere(wo, wi) {
            abs_cos_theta(wi) * std::f64::consts::FRAC_1_PI
        } else {
            0.0
        }
    }

    fn sample_f(&self, _point: Point3<f64>, wo: Vector3<f64>) -> (Vector3<f64>, f64, Vector3<f64>) {
        // todo: check if correct, copied from lambertian
        let mut wi = get_cosine_weighted_in_hemisphere();
        if wo.z < 0.0 {
            wi.z = -wi.z;
        }

        (wi, self.pdf(wo, wi), self.f(wo, wi))
    }
}
