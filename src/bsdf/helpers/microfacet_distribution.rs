use std::f64::consts::{PI, TAU};

use nalgebra::{Point2, Vector3};

use crate::bsdf::helpers::{
    abs_cos_theta, cos_2_phi, cos_2_theta, cos_phi, cos_theta, same_hemisphere, sin_2_phi, sin_phi,
    tan_2_theta, tan_theta,
};
use crate::helpers::spherical_direction;

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

    fn get_sample_visible_area(&self) -> bool;

    fn sample_wh(&self, wo: Vector3<f64>, sample_u: Point2<f64>) -> Vector3<f64>;

    fn pdf(&self, wo: Vector3<f64>, wh: Vector3<f64>) -> f64 {
        if self.get_sample_visible_area() {
            self.d(wh) * self.g1(wo) * wh.dot(&wo).abs() / abs_cos_theta(wo)
        } else {
            self.d(wh) * abs_cos_theta(wh)
        }
    }
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

    fn trowbridge_reinz_sample_11(cos_theta: f64, u1: f64, u2: f64) -> (f64, f64) {
        if cos_theta > 0.9999 {
            let r = (u1 / (1.0 - u1)).sqrt();
            let phi = TAU * u2;
            let slope_x = r * phi.cos();
            let slope_y = r * phi.sin();

            return (slope_x, slope_y);
        }

        let sin_theta = (0.0f64).max(1.0 - cos_theta.powi(2)).sqrt();
        let tan_theta = sin_theta / cos_theta;
        let a = 1.0 / tan_theta;
        let g1 = 2.0 / (1.0 + (1.0 + 1.0 / (a * a)).sqrt());

        // sample slope_x
        let a = 2.0 * u1 / g1 - 1.0;
        let mut tmp = 1.0 / (a * a - 1.0);
        if tmp > 1e10 {
            tmp = 1e10;
        }
        let b = tan_theta;
        let d = (b * b * tmp * tmp - (a * a - b * b) * tmp).max(0.0).sqrt();
        let slope_x_1 = b * tmp - d;
        let slope_x_2 = b * tmp + d;
        let slope_x = if a < 0.0 || slope_x_2 > 1.0 / tan_theta {
            slope_x_1
        } else {
            slope_x_2
        };

        // sample slope_y
        let (s, u2) = if u2 > 0.5 {
            (1.0, 2.0 * (u2 - 0.5))
        } else {
            (-1.0, 2.0 * (0.5 - u2))
        };
        let z = (u2 * (u2 * (u2 * 0.27385 - 0.73369) + 0.46341))
            / (u2 * (u2 * (u2 * 0.093073 + 0.309420) - 1.0) + 0.597999);
        let slope_y = s * z * (1.0 + slope_x * slope_x).sqrt();

        (slope_x, slope_y)
    }

    fn trowbridge_reinz_sample(
        wi: Vector3<f64>,
        alpha_x: f64,
        alpha_y: f64,
        u1: f64,
        u2: f64,
    ) -> Vector3<f64> {
        let wi_stretched = Vector3::new(alpha_x * wi.x, alpha_y * wi.y, wi.z).normalize();
        let (mut slope_x, mut slope_y) =
            Self::trowbridge_reinz_sample_11(cos_theta(wi_stretched), u1, u2);
        let tmp = cos_phi(wi_stretched) * slope_x - sin_phi(wi_stretched) * slope_y;
        slope_y = sin_phi(wi_stretched) * slope_x + cos_phi(wi_stretched) * slope_y;
        slope_x = tmp;

        slope_x *= alpha_x;
        slope_y *= alpha_y;

        Vector3::new(-slope_x, -slope_y, 1.0).normalize()
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

    fn get_sample_visible_area(&self) -> bool {
        self.sample_visible_area
    }

    fn sample_wh(&self, wo: Vector3<f64>, sample_u: Point2<f64>) -> Vector3<f64> {
        ///Vector3f wh;
        //     if (!sampleVisibleArea) {
        //         Float cosTheta = 0, phi = (2 * Pi) * u[1];
        //         if (alphax == alphay) {
        //             Float tanTheta2 = alphax * alphax * u[0] / (1.0f - u[0]);
        //             cosTheta = 1 / std::sqrt(1 + tanTheta2);
        //         } else {
        //             phi =
        //                 std::atan(alphay / alphax * std::tan(2 * Pi * u[1] + .5f * Pi));
        //             if (u[1] > .5f) phi += Pi;
        //             Float sinPhi = std::sin(phi), cosPhi = std::cos(phi);
        //             const Float alphax2 = alphax * alphax, alphay2 = alphay * alphay;
        //             const Float alpha2 =
        //                 1 / (cosPhi * cosPhi / alphax2 + sinPhi * sinPhi / alphay2);
        //             Float tanTheta2 = alpha2 * u[0] / (1 - u[0]);
        //             cosTheta = 1 / std::sqrt(1 + tanTheta2);
        //         }
        //         Float sinTheta =
        //             std::sqrt(std::max((Float)0., (Float)1. - cosTheta * cosTheta));
        //         wh = SphericalDirection(sinTheta, cosTheta, phi);
        //         if (!SameHemisphere(wo, wh)) wh = -wh;
        //     } else {
        //         bool flip = wo.z < 0;
        //         wh = TrowbridgeReitzSample(flip ? -wo : wo, alphax, alphay, u[0], u[1]);
        //         if (flip) wh = -wh;
        //     }
        //     return wh;
        if !self.sample_visible_area {
            let mut cos_theta = 0.0;
            let mut phi = 2.0 * PI * sample_u.x;

            if self.alpha_x == self.alpha_y {
                let tan_theta_2 = self.alpha_x * self.alpha_x * sample_u.x / (1.0 - sample_u.x);
                cos_theta = 1.0 / (1.0 + tan_theta_2).sqrt();
            } else {
                phi =
                    (self.alpha_x / self.alpha_y * (2.0 * PI * sample_u.y + 0.5 * PI).tan()).atan();
                if sample_u.y > 0.5 {
                    phi += PI;
                }
                let sin_phi = phi.sin();
                let cos_phi = phi.cos();
                let alpha_x2 = self.alpha_x * self.alpha_x;
                let alpha_y2 = self.alpha_y * self.alpha_y;
                let alpha2 = 1.0 / (cos_phi * cos_phi / alpha_x2 + sin_phi * sin_phi / alpha_y2);
                let tan_theta_2 = alpha2 * sample_u.x / (1.0 - sample_u.x);
                cos_theta = 1.0 / (1.0 + tan_theta_2).sqrt();
            }

            let sin_theta = (0.0f64).max(1.0 - cos_theta * cos_theta).sqrt();
            let wh = spherical_direction(sin_theta, cos_theta, phi);
            if !same_hemisphere(wo, wh) {
                -wh
            } else {
                wh
            }
        } else {
            let flip = wo.z < 0.0;
            let wh = TrowbridgeReitzDistribution::trowbridge_reinz_sample(
                if flip { -wo } else { wo },
                self.alpha_x,
                self.alpha_y,
                sample_u.x,
                sample_u.y,
            );
            if flip {
                -wh
            } else {
                wh
            }
        }
    }
}
