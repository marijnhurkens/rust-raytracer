use nalgebra::{Point2, Point3, Vector3};
use num_traits::Zero;

use crate::bsdf::helpers::{get_cosine_weighted_in_hemisphere, same_hemisphere};
use crate::helpers::vector_reflect;

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
        if !same_hemisphere(wo, wi) {
            return 0.0;
        }

        let wh = (wi + wo).normalize();

        self.distribution.pdf(wo, wh) / (4.0 * wo.dot(&wh))
    }

    fn sample_f(&self, sample_2: Point3<f64>, wo: Vector3<f64>) -> (Vector3<f64>, f64, Vector3<f64>) {
        ///   // Sample microfacet orientation $\wh$ and reflected direction $\wi$
        //     if (wo.z == 0) return 0.;
        //     Vector3f wh = distribution->Sample_wh(wo, u);
        //     if (Dot(wo, wh) < 0) return 0.;   // Should be rare
        //     *wi = Reflect(wo, wh);
        //     if (!SameHemisphere(wo, *wi)) return Spectrum(0.f);
        //
        //     // Compute PDF of _wi_ for microfacet reflection
        //     *pdf = distribution->Pdf(wo, wh) / (4 * Dot(wo, wh));
        //     return f(wo, *wi);
        if wo.z == 0.0 {
            return (Vector3::zeros(), 0.0, Vector3::zeros());
        }

        let wh = self.distribution.sample_wh(wo, Point2::new(sample_2.x, sample_2.y));

        if wo.dot(&wh) < 0.0 {
            return (Vector3::zeros(), 0.0, Vector3::zeros());
        }

        let wi = vector_reflect(wo, wh);
        if !same_hemisphere(wo, wi) {
            return (Vector3::zeros(), 0.0, Vector3::zeros());
        }

        let pdf = self.distribution.pdf(wo, wh) / (4.0 * wo.dot(&wh));

        (wi, pdf, self.f(wo, wi))
    }
}
