use super::helpers::fresnel::{FresnelDielectric, FresnelTrait};
use super::helpers::microfacet_distribution::{
    MicrofacetDistribution, TrowbridgeReitzDistribution,
};
use crate::bsdf::helpers::{abs_cos_theta, cos_theta, same_hemisphere};
use crate::bsdf::microfacet_reflection::MicrofacetReflection;
use crate::bsdf::{BXDFtrait, BXDFTYPES};
use crate::helpers::{face_forward, refract, vector_reflect};
use nalgebra::{Point2, Vector3};
use num_traits::Zero;

#[derive(Debug, Copy, Clone)]
pub struct MicrofacetTransmission {
    transmission_color: Vector3<f64>,
    distribution: TrowbridgeReitzDistribution,
    fresnel: FresnelDielectric,
    eta_a: f64,
    eta_b: f64,
}

impl MicrofacetTransmission {
    pub fn new(
        transmission_color: Vector3<f64>,
        distribution: TrowbridgeReitzDistribution,
        fresnel: FresnelDielectric,
        eta_a: f64,
        eta_b: f64,
    ) -> Self {
        MicrofacetTransmission {
            transmission_color,
            distribution,
            fresnel,
            eta_a,
            eta_b,
        }
    }
}

impl BXDFtrait for MicrofacetTransmission {
    fn get_type_flags(&self) -> BXDFTYPES {
        BXDFTYPES::TRANSMISSION | BXDFTYPES::GLOSSY
    }

    fn f(&self, wo: Vector3<f64>, wi: Vector3<f64>) -> Vector3<f64> {
        if same_hemisphere(wo, wi) {
            return Vector3::zeros();
        }

        let cos_theta_o = abs_cos_theta(wo);
        let cos_theta_i = abs_cos_theta(wi);
        if cos_theta_o == 0.0 || cos_theta_i == 0.0 {
            return Vector3::zeros();
        }

        // Compute $\wh$ from $\wo$ and $\wi$ for microfacet transmission
        let eta = if cos_theta(wo) > 0.0 {
            self.eta_b / self.eta_a
        } else {
            self.eta_a / self.eta_b
        };
        let mut wh = (wo + wi * eta).normalize();
        if wh.z < 0.0 {
            wh = -wh;
        }

        // same side?
        if wo.dot(&wh) * wi.dot(&wh) > 0.0 {
            return Vector3::zeros();
        }

        let f = self.fresnel.evaluate(wo.dot(&wh));

        let sqrt_denom = wo.dot(&wh) + eta * wi.dot(&wh);
        let factor = 1.0 / eta;

        (1.0 - f)
            * self.transmission_color
            * (self.distribution.d(wh)
                * self.distribution.g(wo, wi)
                * eta
                * eta
                * wi.dot(&wh).abs()
                * wo.dot(&wh).abs()
                * factor
                * factor
                / (cos_theta_i * cos_theta_o * sqrt_denom * sqrt_denom))
                .abs()
    }

    fn pdf(&self, wo: Vector3<f64>, wi: Vector3<f64>) -> f64 {
        if same_hemisphere(wo, wi) {
            return 0.0;
        }

        let eta = if cos_theta(wo) > 0.0 {
            self.eta_b / self.eta_a
        } else {
            self.eta_a / self.eta_b
        };

        let wh = (wo + wi * eta).normalize();

        if wo.dot(&wh) * wi.dot(&wh) > 0.0 {
            return 0.0;
        }
        let sqrt_denom = wo.dot(&wh) + eta * wi.dot(&wh);
        let dwh_dwi = ((eta * eta * wi.dot(&wh)) / (sqrt_denom * sqrt_denom)).abs();

        self.distribution.pdf(wo, wh) / dwh_dwi
    }

    fn sample_f(
        &self,
        sample_2: Point2<f64>,
        wo: Vector3<f64>,
    ) -> (Vector3<f64>, f64, Vector3<f64>) {
        if wo.z == 0.0 {
            return (Vector3::zeros(), 0.0, Vector3::zeros());
        }

        let wh = self.distribution.sample_wh(wo, sample_2);

        if wo.dot(&wh) < 0.0 {
            return (Vector3::zeros(), 0.0, Vector3::zeros());
        }

        let eta = if cos_theta(wo) > 0.0 {
            self.eta_a / self.eta_b
        } else {
            self.eta_b / self.eta_a
        };

        let refract_result = refract(wo, wh, eta);
        let wi = if let Some(wi) = refract_result {
            wi
        } else {
            return (Vector3::zeros(), 0.0, Vector3::zeros());
        };

        let pdf = self.distribution.pdf(wo, wi);

        let f = self.f(wo, wi);

        (wi, pdf, f)
    }
}
