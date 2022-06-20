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
    let d = concentric_sample_disk();
    let z = f64::max(0.0, 1.0 - d.x * d.x - d.y * d.y).sqrt();

    Vector3::new(d.x, d.y, z)
}

fn concentric_sample_disk() -> Point2<f64> {
    let mut rng = thread_rng();

    let u_offset = Point2::new(rng.gen::<f64>(), rng.gen::<f64>()) * 2.0 - Vector2::new(1.0, 1.0);

    if u_offset.x == 0.0 && u_offset.y == 0.0 {
        return Point2::new(0.0, 0.0);
    }

    let (theta, r);
    if u_offset.x.abs() > u_offset.y.abs() {
        r = u_offset.x;
        theta = FRAC_PI_4 * (u_offset.y / u_offset.x);
    } else {
        r = u_offset.y;
        theta = FRAC_PI_2 - FRAC_PI_4 * (u_offset.x / u_offset.y);
    }

    r * Point2::new(theta.cos(), theta.sin())
}
