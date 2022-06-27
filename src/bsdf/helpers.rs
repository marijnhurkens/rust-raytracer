pub mod fresnel;
pub mod microfacet_distribution;

use std::f64::consts::{FRAC_PI_2, FRAC_PI_4};

use nalgebra::{Point2, Vector2, Vector3};
use rand::thread_rng;
use rand::Rng;

pub fn cos_theta(a: Vector3<f64>) -> f64 {
    a.z
}
pub fn cos_2_theta(a: Vector3<f64>) -> f64 {
    a.z * a.z
}
pub fn abs_cos_theta(a: Vector3<f64>) -> f64 {
    a.z.abs()
}
pub fn sin_2_theta(w: Vector3<f64>) -> f64 {
    (1.0 - cos_2_theta(w).max(0.0))
}
pub fn sin_theta(w: Vector3<f64>) -> f64 {
    sin_2_theta(w).sqrt()
}
pub fn tan_theta(w: Vector3<f64>) -> f64 {
    sin_theta(w) / cos_theta(w)
}
pub fn tan_2_theta(w: Vector3<f64>) -> f64 {
    sin_2_theta(w) / cos_2_theta(w)
}
pub fn cos_phi(w: Vector3<f64>) -> f64 {
    let sin_theta = sin_theta(w);
    if sin_theta == 0.0 {
        1.0
    } else {
        (w.x / sin_theta).clamp(-1.0, 1.0)
    }
}
pub fn sin_phi(w: Vector3<f64>) -> f64 {
    let sin_theta = sin_theta(w);
    if sin_theta == 0.0 {
        1.0
    } else {
        (w.y / sin_theta).clamp(-1.0, 1.0)
    }
}
pub fn cos_2_phi(w: Vector3<f64>) -> f64 {
    cos_phi(w) * cos_phi(w)
}
pub fn sin_2_phi(w: Vector3<f64>) -> f64 {
    sin_phi(w) * sin_phi(w)
}

pub fn same_hemisphere(a: Vector3<f64>, b: Vector3<f64>) -> bool {
    a.z * b.z > 0.0
}

pub fn get_cosine_weighted_in_hemisphere() -> Vector3<f64> {
    let d = crate::helpers::concentric_sample_disk();
    let z = f64::max(0.0, 1.0 - d.x * d.x - d.y * d.y).sqrt();

    Vector3::new(d.x, d.y, z)
}
