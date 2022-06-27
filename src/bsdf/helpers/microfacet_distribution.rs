use std::f64::consts::PI;

use nalgebra::{Point2, Vector3};

use crate::bsdf::helpers::{cos_2_phi, cos_2_theta, sin_2_phi, tan_2_theta, tan_theta};

// todo: create enum
pub trait MicrofacetDistribution {
    fn roughness_to_alpha(roughness: f64) -> f64;

    fn d(&self, wh: Vector3<f64>) -> f64;

    fn lambda(&self, w: Vector3<f64>) -> f64;

    fn g1(&self, w: Vector3<f64>) -> f64 {
        1.0 / (1.0 + self.lambda(w))
    }

    fn g(&self, wo: Vector3<f64>, wi: Vector3<f64>) -> f64 {
        1.0 / (1.0 + self.lambda(wo) + self.lambda(wi))
    }

    // unused?
    //fn sample_wh(wo: Vector3<f64>, sample_u: Point2<f64>) -> Vector3<f64>;
    //fn pdf(wo: &Vector3<f64>, wh: &Vector3<f64>) -> f64;
}

#[derive(Debug, Copy, Clone)]
pub struct TrowbridgeReitzDistribution {
    alpha_x: f64,
    alpha_y: f64,
    sample_visible_area: bool,
}

impl TrowbridgeReitzDistribution {
    pub fn new(alpha_x: f64, alpha_y: f64, sample_visible_area: bool) -> Self {
        TrowbridgeReitzDistribution {
            alpha_x,
            alpha_y,
            sample_visible_area,
        }
    }
}

impl MicrofacetDistribution for TrowbridgeReitzDistribution {
    fn roughness_to_alpha(roughness: f64) -> f64 {
        let roughness = roughness.max(1.0e-3);
        let x = roughness.ln();
        1.62142
            + 0.819955 * x
            + 0.1734 * x * x
            + 0.0171201 * x * x * x
            + 0.000640711 * x * x * x * x
    }

    fn d(&self, wh: Vector3<f64>) -> f64 {
        let tan_2_theta = tan_2_theta(wh);
        if tan_2_theta.is_infinite() {
            return 0.0;
        }

        let cos_4_theta = cos_2_theta(wh) * cos_2_theta(wh);
        let e = (cos_2_phi(wh) / (self.alpha_x * self.alpha_x)
            + sin_2_phi(wh) / (self.alpha_y * self.alpha_y))
            * tan_2_theta;
        
        1.0 / (PI * self.alpha_x * self.alpha_y * cos_4_theta * (1.0 + e) * (1.0 + e))
    }

    fn lambda(&self, w: Vector3<f64>) -> f64 {
        let abs_tan_theta = tan_theta(w).abs();
        if abs_tan_theta.is_infinite() {
            return 0.0;
        }

        let alpha = (cos_2_phi(w) * self.alpha_x * self.alpha_x
            + sin_2_phi(w) * self.alpha_y * self.alpha_y)
            .sqrt();

        let alpha_2_tan_2_theta = (alpha * abs_tan_theta) * (alpha * abs_tan_theta);

        (-1.0 + (1.0 + alpha_2_tan_2_theta).sqrt()) / 2.0
    }
}
