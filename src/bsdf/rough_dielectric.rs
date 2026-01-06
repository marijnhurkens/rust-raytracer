use nalgebra::{Point2, Vector3};

use crate::bsdf::helpers::fresnel::{FresnelDielectric, FresnelTrait};
use crate::bsdf::helpers::{cos_theta, same_hemisphere};
use crate::bsdf::microfacet_reflection::MicrofacetReflection;
use crate::bsdf::microfacet_transmission::MicrofacetTransmission;
use crate::bsdf::specular_transmission::TransportMode;
use crate::bsdf::{BXDFtrait, BXDFTYPES};
use crate::renderer::{debug_write_pixel_f64_on_bounce, debug_write_pixel_on_bounce};
use super::helpers::microfacet_distribution::TrowbridgeReitzDistribution;

/// Rough dielectric BSDF that couples reflection and transmission via Fresnel.
///
/// This is meant to avoid hard/unnatural seams when an "uber" material combines
/// independent reflection and transmission lobes.
#[derive(Debug, Copy, Clone)]
pub struct RoughDielectric {
    r: MicrofacetReflection,
    t: MicrofacetTransmission,
    eta_a: f64,
    eta_b: f64,
    fresnel: FresnelDielectric,
}

impl RoughDielectric {
    pub fn new(
        reflectance_color: Vector3<f64>,
        transmittance_color: Vector3<f64>,
        distribution: TrowbridgeReitzDistribution,
        eta_a: f64,
        eta_b: f64,
    ) -> Self {
        let fresnel = FresnelDielectric::new(eta_a, eta_b);

        RoughDielectric {
            r: MicrofacetReflection::new(reflectance_color, distribution, fresnel),
            t: MicrofacetTransmission::new(transmittance_color, distribution, fresnel, eta_a, eta_b),
            eta_a,
            eta_b,
            fresnel,
        }
    }

    fn fresnel_at_wo(&self, wo: Vector3<f64>) -> f64 {
        // Use a smooth Fresnel estimate based on the macrosurface normal.
        // Coupling is what matters here; the microfacet lobes use wh in their own eval.
        self.fresnel.evaluate(cos_theta(wo).abs())
    }
}

impl BXDFtrait for RoughDielectric {
    fn get_type_flags(&self) -> BXDFTYPES {
        // This bxdf can produce both reflection and transmission directions.
        BXDFTYPES::GLOSSY | BXDFTYPES::REFLECTION | BXDFTYPES::TRANSMISSION
    }

    fn f(&self, wo: Vector3<f64>, wi: Vector3<f64>) -> Vector3<f64> {
        let f = if same_hemisphere(wo, wi) {
            self.r.f(wo, wi)
        } else {
            self.t.f(wo, wi)
        };

        f
    }

    fn pdf(&self, wo: Vector3<f64>, wi: Vector3<f64>) -> f64 {
        let fr = self.fresnel_at_wo(wo);


        if same_hemisphere(wo, wi) {
            fr * self.r.pdf(wo, wi)
        } else {
            (1.0 - fr) * self.t.pdf(wo, wi)
        }
    }

    fn sample_f(&self, sample_2: Point2<f64>, wo: Vector3<f64>) -> (Vector3<f64>, f64, Vector3<f64>) {
        let fr = self.fresnel_at_wo(wo);



       // Use sample_2.x to choose reflection vs transmission,
        //and remap to [0,1) for the selected lobe.
        if sample_2.x < fr {
            let u = Point2::new((sample_2.x / fr).min(1.0 - f64::EPSILON), sample_2.y);
            let (wi, _pdf_lobe, f) = self.r.sample_f(u, wo);

            // Mixture pdf: p = fr * p_r(wi)
            let pdf = self.pdf(wo, wi);
            (wi, pdf, f)
        } else {
            let one_minus = (1.0 - fr).max(1e-12);
            let u = Point2::new(
                ((sample_2.x - fr) / one_minus).min(1.0 - f64::EPSILON),
                sample_2.y,
            );
            let (wi, _pdf_lobe, f) = self.t.sample_f(sample_2, wo);

            // Mixture pdf: p = (1-fr) * p_t(wi)
            let pdf = self.pdf(wo, wi);
            (wi, pdf, f)
        }
    }
}

