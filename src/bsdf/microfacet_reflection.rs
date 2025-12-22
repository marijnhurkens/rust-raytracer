use nalgebra::{Point2, Point3, Vector3};
use num_traits::Zero;

use crate::bsdf::helpers::{get_cosine_weighted_in_hemisphere, same_hemisphere};
use crate::helpers::{face_forward, vector_reflect};
use crate::renderer::{debug_write_pixel_f64_on_bounce, debug_write_pixel_on_bounce};

use super::helpers::abs_cos_theta;
use super::helpers::fresnel::{FresnelDielectric, FresnelTrait};
use super::helpers::microfacet_distribution::{
    MicrofacetDistribution, TrowbridgeReitzDistribution,
};
use super::{BXDFtrait, BXDFTYPES};

#[derive(Debug, Copy, Clone)]
pub struct MicrofacetReflection {
    reflectance_color: Vector3<f64>,
    distribution: TrowbridgeReitzDistribution,
    fresnel: FresnelDielectric,
}

impl MicrofacetReflection {
    pub fn new(
        reflectance_color: Vector3<f64>,
        distribution: TrowbridgeReitzDistribution,
        fresnel: FresnelDielectric,
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

        if wh.is_zero() {
            return Vector3::zeros();
        }

        let wh = wh.normalize();

        let f = self.fresnel.evaluate(wi.dot(&face_forward(wh, Vector3::new(0.0, 0.0, 1.0))).abs());

        self.reflectance_color * self.distribution.d(wh) * self.distribution.g(wo, wi) * f
            / (4.0 * cos_theta_i * cos_theta_o)
    }

    fn pdf(&self, wo: Vector3<f64>, wi: Vector3<f64>) -> f64 {
        if !same_hemisphere(wo, wi) {
            return 0.0;
        }

        let wh = (wo + wi).normalize();

        self.distribution.pdf(wo, wh) / (4.0 * wo.dot(&wh))
    }

    fn sample_f(
        &self,
        sample_2: Point2<f64>,
        wo: Vector3<f64>,
    ) -> (Vector3<f64>, f64, Vector3<f64>) {
        if wo.z == 0.0 {
            return (Vector3::zeros(), 0.0, Vector3::zeros());
        }

        let wh = self
            .distribution
            .sample_wh(wo, sample_2);

        if wo.dot(&wh) < 0.0 {
            return (Vector3::zeros(), 0.0, Vector3::zeros());
        }

        let wi = vector_reflect(wo, wh);
        if !same_hemisphere(wo, wi) {
            return (wi, 0.0, Vector3::zeros());
        }

        let pdf = self.distribution.pdf(wo, wh) / (4.0 * wo.dot(&wh));

        let f = self.f(wo, wi);

        (wi, pdf, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Vector3;

    #[test]
    fn test_microfacet_reflection_f() {
        let reflectance = Vector3::new(0.9, 0.9, 0.9);
        let distribution = TrowbridgeReitzDistribution::new(0.5, 0.5, true);
        let fresnel = FresnelDielectric::new(1.0, 1.5);
        let bsdf = MicrofacetReflection::new(reflectance, distribution, fresnel);

        // Test normal incidence
        let wo = Vector3::new(0.0, 0.0, 1.0);
        let wi = Vector3::new(0.0, 0.0, 1.0);

        let f = bsdf.f(wo, wi);

        assert!(f.x >= 0.0);
        assert!(f.y >= 0.0);
        assert!(f.z >= 0.0);
        assert!(!f.x.is_nan());
        assert!(!f.y.is_nan());
        assert!(!f.z.is_nan());

        // Test with some angle (reflection configuration)
        let wo = Vector3::new(0.6, 0.0, 0.8).normalize();
        let wi = Vector3::new(-0.6, 0.0, 0.8).normalize();
        let f = bsdf.f(wo, wi);

        assert!(f.x >= 0.0);
        assert!(!f.x.is_nan());
    }
}
