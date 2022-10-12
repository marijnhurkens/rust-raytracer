use nalgebra::{Point3, Vector3};

use crate::bsdf::helpers::fresnel::{Fresnel, FresnelDielectric, FresnelTrait};
use crate::bsdf::helpers::{abs_cos_theta, cos_theta};
use crate::bsdf::{BXDFtrait, BXDFTYPES};
use crate::helpers::{face_forward, refract};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TransportMode {
    Radiance,
    Other,
}

#[derive(Debug, Clone, Copy)]
pub struct SpecularTransmission {
    refraction_color: Vector3<f64>,
    fresnel: Fresnel,
    eta_a: f64,
    eta_b: f64,
    mode: TransportMode,
}

impl SpecularTransmission {
    pub fn new(
        refraction_color: Vector3<f64>,
        eta_a: f64,
        eta_b: f64,
        mode: TransportMode,
    ) -> Self {
        SpecularTransmission {
            refraction_color,
            fresnel: Fresnel::Dielectric(FresnelDielectric::new(eta_a, eta_b)),
            eta_a,
            eta_b,
            mode,
        }
    }
}

impl BXDFtrait for SpecularTransmission {
    fn get_type_flags(&self) -> BXDFTYPES {
        BXDFTYPES::REFRACTION | BXDFTYPES::SPECULAR
    }

    fn f(&self, _wo: Vector3<f64>, _wi: Vector3<f64>) -> Vector3<f64> {
        Vector3::zeros()
    }

    fn pdf(&self, _wo: Vector3<f64>, _wi: Vector3<f64>) -> f64 {
        1.0
    }

    fn sample_f(&self, _point: Point3<f64>, wo: Vector3<f64>) -> (Vector3<f64>, f64, Vector3<f64>) {
        let (eta_i, eta_t) = if cos_theta(wo) > 0.0 {
            (self.eta_a, self.eta_b)
        } else {
            (self.eta_b, self.eta_a)
        };

        let normal = face_forward(Vector3::new(0.0, 0.0, 1.0), wo);
        let wi = if let Some(wi) = refract(wo, normal, eta_i / eta_t) {
            wi
        } else {
            return (Vector3::zeros(), 0.0, Vector3::zeros());
        };

        let fresnel_eval = self.fresnel.evaluate(cos_theta(wi));
        let mut ft = self
            .refraction_color
            .component_mul(&(Vector3::repeat(1.0) - Vector3::repeat(fresnel_eval)));

        if self.mode == TransportMode::Radiance {
            ft *= (eta_i * eta_i) / (eta_t * eta_t);
        }

        (wi, 1.0, ft / abs_cos_theta(wi))
    }
}
